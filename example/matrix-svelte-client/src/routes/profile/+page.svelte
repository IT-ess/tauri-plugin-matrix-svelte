<script lang="ts">
	import ProfileDetails from '$lib/components/profiles/profile-details.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<ProfileDetails profile={data.profile} />
