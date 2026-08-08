<script lang="ts">
	import PartPreview from '$lib/PartPreview.svelte';
	import type { PartGroup } from '$lib/avatarParts';

	type Props = {
		part: PartGroup;
		value: string;
		onSelect: (optionId: string) => void;
	};

	let { part, value, onSelect }: Props = $props();
	let container: HTMLDivElement;
	let open = $state(false);
	let selectedOption = $derived(
		part.options.find((option) => option.id === value) ?? part.options[0]
	);

	let listboxId = $derived(`character-${part.id}-options`);

	function selectOption(optionId: string): void {
		onSelect(optionId);
		open = false;
	}

	function handleWindowClick(event: MouseEvent): void {
		if (open && !container.contains(event.target as Node)) {
			open = false;
		}
	}

	function handleKeydown(event: KeyboardEvent): void {
		if (event.key === 'Escape') {
			open = false;
		}
	}
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleKeydown} />

<div class="relative" bind:this={container}>
	<button
		type="button"
		aria-haspopup="listbox"
		aria-expanded={open}
		aria-controls={listboxId}
		onclick={() => (open = !open)}
		class={[
			'flex w-full items-center gap-2 rounded-xl border bg-[#faf8f4] p-1.5 text-left transition',
			'focus:ring-2 focus:ring-[#d8688d] focus:outline-none',
			open
				? 'border-[#d8688d] bg-[#fff6f8] text-[#30233b]'
				: 'border-[#e0d8d0] text-[#514655] hover:border-[#bfaebb]'
		]}
	>
		<span class="size-10 shrink-0 overflow-hidden rounded-lg bg-[#2d213b]">
			<PartPreview part={part.id} option={selectedOption.id} />
		</span>
		<span class="min-w-0 flex-1 truncate text-xs font-semibold sm:text-sm"
			>{selectedOption.name}</span
		>
		<svg
			class={['size-4 shrink-0 text-gray-400 transition-transform', open && 'rotate-180']}
			viewBox="0 0 20 20"
			fill="currentColor"
			aria-hidden="true"
		>
			<path
				fill-rule="evenodd"
				d="M5.22 7.22a.75.75 0 0 1 1.06 0L10 10.94l3.72-3.72a.75.75 0 1 1 1.06 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L5.22 8.28a.75.75 0 0 1 0-1.06Z"
				clip-rule="evenodd"
			/>
		</svg>
	</button>

	{#if open}
		<div
			id={listboxId}
			role="listbox"
			aria-label={`${part.name} options`}
			class="absolute top-full right-0 left-0 z-30 mt-1.5 max-h-64 min-w-40 space-y-1 overflow-y-auto rounded-xl border border-[#ded4dd] bg-white p-1.5 shadow-2xl shadow-[#392844]/20"
		>
			{#each part.options as option (option.id)}
				<button
					type="button"
					role="option"
					aria-selected={value === option.id}
					onclick={() => selectOption(option.id)}
					class={[
						'flex w-full items-center gap-2 rounded-lg p-1 text-left transition',
						'focus:ring-2 focus:ring-[#d8688d] focus:outline-none',
						value === option.id
							? 'bg-[#f9e9ee] text-[#5f2940]'
							: 'text-[#625766] hover:bg-[#f7f3ee]'
					]}
				>
					<span class="size-10 shrink-0 overflow-hidden rounded-md bg-[#2d213b]">
						<PartPreview part={part.id} option={option.id} />
					</span>
					<span class="min-w-0 flex-1 truncate text-sm font-medium">{option.name}</span>
					{#if value === option.id}
						<svg class="size-4 shrink-0 text-[#d45f87]" viewBox="0 0 20 20" fill="currentColor">
							<path
								fill-rule="evenodd"
								d="M16.7 5.3a1 1 0 0 1 0 1.4l-8 8a1 1 0 0 1-1.4 0l-4-4a1 1 0 1 1 1.4-1.4L8 12.58l7.3-7.3a1 1 0 0 1 1.4.02Z"
								clip-rule="evenodd"
							/>
						</svg>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>
