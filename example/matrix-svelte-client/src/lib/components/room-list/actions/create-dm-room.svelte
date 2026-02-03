<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';

	import { buttonVariants } from '$lib/components/ui/button';

	import SearchProfile from '$lib/components/profiles/search-profile.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		actionCreateDMRoomOpen: boolean;
	};

	let { actionCreateDMRoomOpen = $bindable(false) }: Props = $props();
</script>

<Dialog.Root bind:open={actionCreateDMRoomOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{m.search_profiles_add_new()}</Dialog.Title>
			<Dialog.Description>
				{m.search_profiles_subtitle()}
			</Dialog.Description>
		</Dialog.Header>
		<div class="mx-4 h-72 overflow-y-hidden pb-20">
			<SearchProfile bind:parentOpen={actionCreateDMRoomOpen} />
		</div>
		<Dialog.Footer>
			<Dialog.Close class={buttonVariants({ variant: 'outline' })}>{m.button_cancel()}</Dialog.Close
			>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
