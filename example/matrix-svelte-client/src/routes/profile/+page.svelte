<script lang="ts">
	import ProfileDetails from '$lib/components/profiles/profile-details.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<ProfileDetails matrixId={data.matrixUserId} avatar={data.avatar} name={data.name} />
