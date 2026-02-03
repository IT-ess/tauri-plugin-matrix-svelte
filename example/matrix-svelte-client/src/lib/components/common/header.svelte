<script lang="ts">
	import ProfileMenu from '$lib/components/common/profile-menu.svelte';
	import * as m from '$lib/paraglide/messages';

	let {
		currentInvitesNumber
	}: {
		currentInvitesNumber: number;
	} = $props();
</script>

<header
	class="h-header-safe pt-safe sticky top-0 right-0 left-0 z-20 flex flex-1 justify-between bg-white shadow-2xl"
>
	<div class="ml-5">
		<p class="ml-1 font-serif text-2xl">{m.header_messages()}</p>
	</div>
	<div class="mr-4 flex items-center gap-2 pb-2">
		<ProfileMenu {currentInvitesNumber} />
	</div>
</header>
