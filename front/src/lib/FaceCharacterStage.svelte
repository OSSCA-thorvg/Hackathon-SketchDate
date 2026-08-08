<script lang="ts">
	import CharacterRenderer from '$lib/CharacterRenderer.svelte';
	import type { PartSelection } from '$lib/avatarParts';
	import type { RigPose } from '$lib/characterRig';

	type Props = {
		pose: RigPose | null;
		parts: PartSelection;
		running: boolean;
		modelReady: boolean;
		cameraStatus: string;
		faceCount: number;
		videoElement?: HTMLVideoElement;
	};

	let {
		pose,
		parts,
		running,
		modelReady,
		cameraStatus,
		faceCount,
		videoElement = $bindable()
	}: Props = $props();

	let renderError = $state('');
</script>

<div
	class="relative mx-auto aspect-[4/5] w-4/5 max-w-2xl overflow-hidden rounded-2xl border border-gray-800 bg-black shadow-2xl shadow-black/40"
>
	<video
		bind:this={videoElement}
		autoplay
		muted
		playsinline
		aria-hidden="true"
		class="pointer-events-none absolute size-px opacity-0"
	></video>

	<CharacterRenderer
		{pose}
		{parts}
		label="Custom avatar following the detected face"
		class="pointer-events-none absolute inset-0 block h-full w-full -scale-x-100 object-contain"
		onerror={(error) => (renderError = error.message)}
	/>

	{#if !running}
		<div class="absolute inset-0 grid place-items-center bg-gray-950/95 px-6 text-center">
			<div class="flex flex-col items-center">
				{#if !modelReady}
					<svg
						class="mb-4 size-8 animate-spin text-gray-400"
						viewBox="0 0 24 24"
						fill="none"
						aria-hidden="true"
					>
						<circle
							class="opacity-25"
							cx="12"
							cy="12"
							r="10"
							stroke="currentColor"
							stroke-width="4"
						/>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 0 1 8-8v4a4 4 0 0 0-4 4H4Z"
						/>
					</svg>
					<p class="text-sm text-gray-400">Loading face landmark model…</p>
				{:else}
					<div class="mb-4 grid size-14 place-items-center rounded-full bg-gray-800">
						<svg
							class="size-6 text-gray-300"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.75"
							aria-hidden="true"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="m15.75 10.5 4.72-4.72a.75.75 0 0 1 1.28.53v11.38a.75.75 0 0 1-1.28.53l-4.72-4.72M4.5 18.75h9A2.25 2.25 0 0 0 15.75 16.5v-9A2.25 2.25 0 0 0 13.5 5.25h-9A2.25 2.25 0 0 0 2.25 7.5v9a2.25 2.25 0 0 0 2.25 2.25Z"
							/>
						</svg>
					</div>

					{#if cameraStatus === 'Starting camera…'}
						<p class="font-medium text-gray-200">Starting camera…</p>
						<p class="mt-1 text-sm text-gray-500">Allow camera access when prompted.</p>
					{:else}
						<p class="font-medium text-gray-200">Camera unavailable</p>
						<p class="mt-1 text-sm text-gray-500">{cameraStatus}</p>
					{/if}
				{/if}
			</div>
		</div>
	{/if}

	{#if running}
		<div
			class="pointer-events-none absolute top-4 left-4 flex items-center gap-2 rounded-full bg-black/60 px-3 py-1.5 text-xs font-medium text-white backdrop-blur-sm"
		>
			<span
				class="size-2 rounded-full"
				class:bg-emerald-400={faceCount > 0}
				class:bg-amber-400={faceCount <= 0}
			></span>
			{faceCount > 0 ? 'Face detected' : 'Searching'}
		</div>
	{/if}

	{#if renderError}
		<div
			class="pointer-events-none absolute right-4 bottom-4 max-w-72 rounded-lg bg-red-950/90 px-3 py-2 text-xs text-red-200"
			role="status"
		>
			Character renderer unavailable: {renderError}
		</div>
	{/if}
</div>
