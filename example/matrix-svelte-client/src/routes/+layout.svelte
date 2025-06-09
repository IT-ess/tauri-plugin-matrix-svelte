<script lang="ts">
	import '../app.css';
	import { goto } from '$app/navigation';
	let { children, data } = $props();

	$effect(() => {
		if (data.loginStore.state.state == 'awaitingForLogin') {
			goto('/login');
		} else {
			data.loginStore.state.state = 'loggedIn';
		}
	});
</script>

{@render children()}

<p>
	Current status: {data.roomsCollection.state.status.status}. Message: {data.roomsCollection.state
		.status.message}
</p>
