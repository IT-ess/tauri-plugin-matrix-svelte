<script lang="ts">
	import { Avatar as AvatarPrimitive } from 'bits-ui';
	import { cn } from '$lib/utils.svelte.js';

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: AvatarPrimitive.ImageProps = $props();
</script>

<AvatarPrimitive.Image
	bind:ref
	loading="lazy"
	data-slot="avatar-image"
	class={cn('aspect-square size-full rounded-full object-cover', className)}
	{...restProps}
/>
