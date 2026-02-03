<script lang="ts">
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { m } from '$lib/paraglide/messages';
	import { gotoRoomsList } from '$lib/utils.svelte';
	import { createMatrixRequest, submitAsyncRequest } from 'tauri-plugin-matrix-svelte-api';

	let {
		open = $bindable(false),
		roomId,
		isDirect
	}: { open: boolean; roomId: string; isDirect: boolean } = $props();

	const handleLeaveRoom = async () => {
		const request = createMatrixRequest.leaveRoom({ roomId });
		await submitAsyncRequest(request);
		await gotoRoomsList(isDirect ? 'dm' : 'groups');
	};
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{m.logout_are_you_sure()}</AlertDialog.Title>
			<AlertDialog.Description>
				{m.leave_room_description()}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel>{m.button_cancel()}</AlertDialog.Cancel>
			<AlertDialog.Action onclick={handleLeaveRoom}>{m.button_confirm()}</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
