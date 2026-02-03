<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { PageData } from './$types';
	import { ChevronLeft } from '@lucide/svelte';
	import NotificationsSwitch from '$lib/components/settings/notifications-switch.svelte';
	import { gotoRoomsList } from '$lib/utils.svelte';

	let { data }: { data: PageData } = $props();
</script>

<div class="bg-background pt-safe flex min-h-screen flex-col px-6 py-8">
	<header class="mt-10 mb-3">
		<button
			onclick={() => gotoRoomsList('dm')}
			class="top-safe-offset-2 hover:bg-accent absolute left-3 flex h-10 w-10 items-center justify-center rounded-full transition-colors"
			aria-label="Go back"
		>
			<ChevronLeft class="text-foreground h-6 w-6" />
		</button>

		<h1 class="mb-2 font-serif text-2xl text-balance">{m.notification_daily_recap()}</h1>
		<p class="text-muted-foreground leading-relaxed">
			{m.notification_daily_recap_description()}
		</p>
	</header>
	<NotificationsSwitch hasPending={data.hasPending} />
</div>
