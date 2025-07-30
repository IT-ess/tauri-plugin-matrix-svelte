use anyhow::anyhow;
use futures_util::stream::StreamExt;
use matrix_sdk::{
    encryption::verification::{
        format_emojis, Emoji, SasState, SasVerification, Verification, VerificationRequest,
        VerificationRequestState,
    },
    ruma::{events::key::verification::VerificationMethod, DeviceId, UserId},
    Client,
};
use tauri::{AppHandle, Emitter, Listener, Runtime};

use crate::{
    matrix::singletons::get_client,
    models::matrix::{
        MatrixSvelteEmitEvent, MatrixSvelteListenEvent, MatrixVerificationEmojis,
        MatrixVerificationResponse,
    },
};

async fn wait_for_confirmation<'a, R: Runtime>(
    sas: SasVerification,
    emoji: [Emoji; 7],
    app_handle: AppHandle<R>,
) -> anyhow::Result<()> {
    let payload = MatrixVerificationEmojis::new(format_emojis(emoji));
    let _ = &app_handle.emit(MatrixSvelteEmitEvent::VerificationStart.as_str(), payload)?;

    let _ = &app_handle.listen(
        MatrixSvelteListenEvent::VerificationResult.as_str(),
        move |event| {
            let sas_clone = sas.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(payload) =
                    serde_json::from_str::<MatrixVerificationResponse>(event.payload())
                {
                    match payload.confirmed {
                        true => sas_clone.confirm().await.unwrap(),
                        false => sas_clone.cancel().await.unwrap(),
                    }
                }
            });
        },
    );
    Ok(())
}

async fn print_devices(user_id: &UserId, client: &Client) {
    println!("Devices of user {user_id}");

    for device in client
        .encryption()
        .get_user_devices(user_id)
        .await
        .unwrap()
        .devices()
    {
        if device.device_id()
            == client
                .device_id()
                .expect("We should be logged in now and know our device id")
        {
            continue;
        }

        println!(
            "   {:<10} {:<30} {:<}",
            device.device_id(),
            device.display_name().unwrap_or("-"),
            if device.is_verified() { "✅" } else { "❌" }
        );
    }
}

async fn sas_verification_handler<R: Runtime>(
    client: Client,
    sas: SasVerification,
    app_handle: AppHandle<R>,
) {
    println!(
        "Starting verification with {} {}",
        &sas.other_device().user_id(),
        &sas.other_device().device_id()
    );
    print_devices(sas.other_device().user_id(), &client).await;
    sas.accept().await.unwrap();

    let mut stream = sas.changes();

    while let Some(state) = stream.next().await {
        match state {
            SasState::KeysExchanged {
                emojis,
                decimals: _,
            } => {
                tauri::async_runtime::spawn(wait_for_confirmation(
                    sas.clone(),
                    emojis
                        .expect("We only support verifications using emojis")
                        .emojis,
                    app_handle.clone(),
                ));
            }
            SasState::Done { .. } => {
                let device = sas.other_device();

                println!(
                    "Successfully verified device {} {} {:?}",
                    device.user_id(),
                    device.device_id(),
                    device.local_trust_state()
                );

                print_devices(sas.other_device().user_id(), &client).await;

                break;
            }
            SasState::Cancelled(cancel_info) => {
                println!(
                    "The verification has been cancelled, reason: {}",
                    cancel_info.reason()
                );

                break;
            }
            SasState::Created { .. }
            | SasState::Started { .. }
            | SasState::Accepted { .. }
            | SasState::Confirmed => (),
        }
    }
}

pub async fn request_verification_handler<R: Runtime>(
    client: Client,
    request: VerificationRequest,
    app_handle: AppHandle<R>,
) {
    println!(
        "Accepting verification request from {}",
        request.other_user_id(),
    );
    request
        .accept()
        .await
        .expect("Can't accept verification request");

    let mut stream = request.changes();

    while let Some(state) = stream.next().await {
        match state {
            VerificationRequestState::Created { .. }
            | VerificationRequestState::Requested { .. }
            | VerificationRequestState::Ready { .. } => (),
            VerificationRequestState::Transitioned { verification } => {
                // We only support SAS verification.
                if let Verification::SasV1(s) = verification {
                    tauri::async_runtime::spawn(sas_verification_handler(
                        client,
                        s,
                        app_handle.clone(),
                    ));
                    break;
                }
            }
            VerificationRequestState::Done | VerificationRequestState::Cancelled(_) => break,
        }
    }
}

pub async fn verify_device<R: Runtime>(
    app_handle: &AppHandle<R>,
    user_id: &UserId,
    device_id: &DeviceId,
) -> anyhow::Result<()> {
    let client = get_client().expect("Client should be defined at this state");
    let device_option = client
        .encryption()
        .get_device(user_id, device_id)
        .await
        .map_err(|e| anyhow!(e))?;

    let verification_methods = vec![VerificationMethod::SasV1];

    let request = if let Some(device) = device_option {
        device
            .request_verification_with_methods(verification_methods)
            .await?
    } else {
        return Err(anyhow!("The provided device ID is not found"));
    };

    let mut stream = request.changes();

    while let Some(state) = stream.next().await {
        match state {
            VerificationRequestState::Created { .. }
            | VerificationRequestState::Requested { .. }
            | VerificationRequestState::Transitioned { .. } => (),
            VerificationRequestState::Ready {
                our_methods: _,
                other_device_data: _,
                their_methods,
            } => {
                if their_methods.contains(&VerificationMethod::SasV1) {
                    if let Some(sas) = request.start_sas().await? {
                        tauri::async_runtime::spawn(sas_verification_handler(
                            client,
                            sas,
                            app_handle.clone(),
                        ));
                        break;
                    };
                } else {
                    request.cancel().await?
                }
            }
            // VerificationRequestState::Transitioned { verification: _ } => {
            //     // // We only support SAS verification.
            //     // if let Verification::SasV1(s) = verification {
            //     //     tauri::async_runtime::spawn(sas_verification_handler(
            //     //         client,
            //     //         s,
            //     //         app_handle.clone(),
            //     //     ));
            //     //     break;
            //     // }
            // }
            VerificationRequestState::Done | VerificationRequestState::Cancelled(_) => break,
        }
    }

    Ok(())
}
