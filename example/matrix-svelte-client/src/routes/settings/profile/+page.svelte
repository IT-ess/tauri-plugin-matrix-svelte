<script lang="ts">
	import ProfileCard from '$lib/components/user-profile/profile-card.svelte';
	import { ChevronLeft } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages.js';
	import { gotoRoomsList } from '$lib/utils.svelte';
</script>

<div
	class="bg-background text-foreground flex h-dvh w-full flex-col items-center justify-baseline p-4"
>
	<header class="mt-10 mb-24">
		<button
			onclick={() => gotoRoomsList('dm')}
			class="top-safe-offset-2 hover:bg-accent absolute left-3 flex h-10 w-10 items-center justify-center rounded-full transition-colors"
			aria-label="Go back"
		>
			<ChevronLeft class="text-foreground h-6 w-6" />
		</button>

		<h1 class="mb-2 text-center text-3xl font-semibold text-balance">{m.user_profile_title()}</h1>
		<p class="text-muted-foreground text-center leading-relaxed">{m.user_profile_subtitle()}</p>
	</header>
	<ProfileCard />
</div>
