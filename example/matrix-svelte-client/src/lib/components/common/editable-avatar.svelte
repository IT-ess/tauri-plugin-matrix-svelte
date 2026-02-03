<script lang="ts">
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { CameraIcon, LoaderIcon } from '@lucide/svelte';
	import { Avatar } from '$lib/components/ui/avatar';
	import { m } from '$lib/paraglide/messages';
	import { cn } from '$lib/utils.svelte';
	import { uploadMedia } from 'tauri-plugin-matrix-svelte-api';

	let {
		setUriCallback,
		previousAvatarUri,
		displayName,
		hideHelpText,
		canEdit
	}: {
		setUriCallback: (uri: string) => Promise<void>;
		previousAvatarUri: string | null | undefined;
		displayName: string;
		hideHelpText?: boolean;
		canEdit: boolean;
	} = $props();

	let fileInput: HTMLInputElement;

	let isLoading = $state(false);
	// svelte-ignore state_referenced_locally
	let currentAvatarUri = $state(previousAvatarUri);

	const handleAvatarChange = async (e: Event) => {
		isLoading = true;
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];

		if (file) {
			let buffer = await file.arrayBuffer();
			currentAvatarUri = await handleUploadMedia(file.type, buffer);
		}
		isLoading = false;
	};

	const handleUploadMedia = async (mimeType: string, data: string | ArrayBuffer) => {
		let uri = await uploadMedia(mimeType, data);
		await setUriCallback(uri);
		return uri;
	};

	const triggerFileInput = () => {
		fileInput?.click();
	};
</script>

<div class="flex flex-col items-center gap-4">
	<div class="relative">
		<Avatar class={cn('ring-primary/20 size-24 shadow-lg ring-4', isLoading ? 'opacity-60' : '')}>
			{#if currentAvatarUri}
				{@render fetchAvatar(currentAvatarUri, displayName)}
			{/if}
			{@render avatarFallback(displayName)}
		</Avatar>
		{#if canEdit}
			<button
				onclick={triggerFileInput}
				disabled={isLoading}
				class="shadow-lg-lg-md hover:shadow-lg-lg-lg bg-primary text-primary-foreground absolute right-0 bottom-0 flex size-8 items-center justify-center rounded-full transition-all duration-200 hover:scale-110 disabled:opacity-50"
				title="Change avatar"
			>
				{#if isLoading}
					<LoaderIcon class="size-4 animate-spin" />
				{:else}
					<CameraIcon class="size-4" />
				{/if}
			</button>
		{/if}
	</div>

	<input
		bind:this={fileInput}
		type="file"
		accept="image/*"
		onchange={handleAvatarChange}
		class="hidden"
	/>
	{#if !hideHelpText}
		<p class="text-muted-foreground text-sm">{m.user_profile_avatar_instructions()}</p>
	{/if}
</div>
