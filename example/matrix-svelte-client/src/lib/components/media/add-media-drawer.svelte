<script lang="ts">
	import * as Drawer from '$lib/components/ui/drawer/index.js';
	import { Camera, CirclePlus, Images, Paperclip, Video } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';

	let {
		openMediaDrawer = $bindable(false),
		handleJoinFile,
		handleOpenGallery,
		handleTakePhoto,
		handleTakeVideo,
		displayTrigger = true
	}: {
		openMediaDrawer: boolean;
		handleTakePhoto: () => void;
		handleTakeVideo: () => void;
		handleOpenGallery: () => void;
		handleJoinFile: () => void;
		displayTrigger: boolean;
	} = $props();
</script>

<Drawer.Root bind:open={openMediaDrawer}>
	{#if displayTrigger}
		<Drawer.Trigger class="mr-2"><CirclePlus class="size-6" /></Drawer.Trigger>
	{/if}
	<Drawer.Content class="pb-safe">
		<Drawer.Footer>
			<Button
				onclick={handleTakePhoto}
				class="w-full justify-baseline text-lg font-normal"
				variant="ghost"
				><Camera class="mr-2 size-5" />{m.room_input_take_picture()}
			</Button>
			<Button
				onclick={handleTakeVideo}
				class="w-full justify-baseline text-lg font-normal"
				variant="ghost"><Video class="mr-2 size-5" />{m.room_input_take_video()}</Button
			>
			<Button
				onclick={handleOpenGallery}
				class="w-full justify-baseline text-lg font-normal"
				variant="ghost"><Images class="mr-2 size-5" />{m.room_input_gallery()}</Button
			>
			<Button
				onclick={handleJoinFile}
				class="w-full justify-baseline text-lg font-normal"
				variant="ghost"><Paperclip class="mr-2 size-5" />{m.room_input_attached_file()}</Button
			>
		</Drawer.Footer>
	</Drawer.Content>
</Drawer.Root>
