use tauri::{Builder, Wry};

pub fn setup_logging(mut tauri_builder: Builder<Wry>) -> Builder<Wry> {
    #[cfg(not(debug_assertions))]
    {
        use time::macros::{format_description, offset};
        use tracing_subscriber::{EnvFilter, fmt::time::OffsetTime};
        let fmt = if cfg!(debug_assertions) {
            format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
        } else {
            format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
            )
        };

        let timer = OffsetTime::new(offset!(+1), fmt);

        let filter = EnvFilter::from_default_env()
            .add_directive(
                "matrix_svelte_client_lib"
                    .parse()
                    .expect("couldn't parse the log filter"),
            )
            .add_directive(
                "matrix_ui_serializable=debug"
                    .parse()
                    .expect("couldn't parse the log filter"),
            )
            .add_directive(
                "matrix_sdk=warn"
                    .parse()
                    .expect("couldn't parse the log filter"),
            )
            .add_directive(
                "matrix_sdk_ui=warn"
                    .parse()
                    .expect("couldn't parse the log filter"),
            );
        let writer = std::io::stderr;

        let builder = tracing_subscriber::fmt()
            .with_file(true)
            .with_line_number(true)
            .with_env_filter(filter)
            .with_target(false)
            .with_timer(timer)
            .with_writer(writer);
        builder.init();
    }
    // only enable devtools on linux in debug mode
    #[cfg(debug_assertions)]
    {
        tauri_builder = tauri_builder.plugin(tauri_plugin_devtools::init());
    }
    tauri_builder
}
