<script lang="ts">
	import ProfileList from '$lib/components/profiles/profile-list.svelte';
	import type { PageProps } from './$types';
	import { ChevronLeft } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { gotoRoomsList } from '$lib/utils.svelte';

	let { data }: PageProps = $props();
</script>

<div class="bg-background pt-safe text-foreground min-h-screen">
	<div class="container mx-auto py-8">
		<button
			onclick={() => gotoRoomsList('dm')}
			class="top-safe-offset-2 hover:bg-accent absolute left-3 flex h-10 w-10 items-center justify-center rounded-full transition-colors"
			aria-label="Go back"
		>
			<ChevronLeft class="text-foreground h-6 w-6" />
		</button>
		<div class="mb-6 text-center">
			<h1 class="text-foreground text-2xl font-bold">{m.profile_menu_contacts()}</h1>
			<p class="text-muted-foreground">{m.contacts_subtitle()}</p>
		</div>

		<div class="bg-card flex h-full w-full flex-col rounded-lg border pb-2 shadow-sm">
			<ProfileList dmRooms={data.dmRooms} openInviteDrawerOnLoad={data.openInviteDrawerOnLoad} />
		</div>
	</div>
</div>
