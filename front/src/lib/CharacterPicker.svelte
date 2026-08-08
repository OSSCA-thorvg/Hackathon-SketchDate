<script lang="ts">
	import CharacterPartSelect from '$lib/CharacterPartSelect.svelte';
	import { PART_GROUPS, type PartKey, type PartSelection } from '$lib/avatarParts';

	type Props = {
		selectedParts: PartSelection;
	};

	let { selectedParts = $bindable() }: Props = $props();

	function choosePart(part: PartKey, optionId: string): void {
		selectedParts = { ...selectedParts, [part]: optionId } as PartSelection;
	}
</script>

<section
	class="rounded-2xl border border-white bg-white/75 p-3 shadow-[0_14px_40px_rgba(64,42,78,.1)] backdrop-blur"
	aria-labelledby="composer-heading"
>
	<div class="mb-3 flex items-center justify-between">
		<div>
			<p class="text-[10px] font-black tracking-[0.16em] text-[#a95573] uppercase">Character kit</p>
			<h2 id="composer-heading" class="font-serif text-lg font-black text-[#30233b]">
				Choose your look
			</h2>
		</div>
		<span class="grid size-7 place-items-center rounded-lg bg-[#f7e4ea] text-xs text-[#a95573]"
			>✦</span
		>
	</div>
	<div class="grid grid-cols-2 gap-2.5">
		{#each PART_GROUPS as part (part.id)}
			<fieldset class={['min-w-0', part.id === 'accessory' && 'col-span-2']}>
				<legend class="mb-1 text-[9px] font-black tracking-[0.14em] text-[#827687] uppercase">
					{part.name}
				</legend>
				<CharacterPartSelect
					{part}
					value={selectedParts[part.id]}
					onSelect={(optionId) => choosePart(part.id, optionId)}
				/>
			</fieldset>
		{/each}
	</div>
</section>
