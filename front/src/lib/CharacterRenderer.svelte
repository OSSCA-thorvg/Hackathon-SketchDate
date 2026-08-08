<script lang="ts">
	import ComposedAvatar from '$lib/ComposedAvatar.svelte';
	import ThorVGRenderer from '$lib/ThorVGRenderer.svelte';
	import type { PartSelection } from '$lib/avatarParts';
	import { NEUTRAL_RIG_POSE, type RigPose } from '$lib/characterRig';
	import type { Matrix } from '@thorvg/webcanvas';
	import { tick } from 'svelte';

	type Props = {
		pose: RigPose | null;
		parts: PartSelection;
		width?: number;
		height?: number;
		label?: string;
		class?: string;
		onerror?: (error: Error) => void;
	};

	let {
		pose,
		parts,
		width = 800,
		height = 1000,
		label = 'Live character',
		class: className = '',
		onerror
	}: Props = $props();

	let characterSvgElement = $state<SVGSVGElement>(undefined!);
	let characterAsset = $state({
		key: 'initial',
		source: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="-160 -210 320 400"></svg>'
	});

	let characterViewBox = $derived.by(() => {
		const viewBoxHeight = 400;
		const viewBoxWidth = viewBoxHeight * (width / height);
		return `${-viewBoxWidth / 2} -210 ${viewBoxWidth} ${viewBoxHeight}`;
	});
	let partsKey = $derived(
		`${parts.head}|${parts.eyes}|${parts.nose}|${parts.mouth}|${parts.accessory}|${characterViewBox}`
	);

	function clamp(value: number, minimum: number, maximum: number): number {
		return Math.min(maximum, Math.max(minimum, value));
	}

	function matrix(e11 = 1, e12 = 0, e13 = 0, e21 = 0, e22 = 1, e23 = 0): Matrix {
		return { e11, e12, e13, e21, e22, e23, e31: 0, e32: 0, e33: 1 };
	}

	function animatedMouthPath(frame: RigPose): {
		commands: readonly number[];
		points: ReadonlyArray<readonly number[]>;
	} {
		if (parts.mouth === 'heart') {
			const bottom = 68 + frame.mouthOpen * 5;
			const corner = 41 - frame.smile * 5;
			return {
				commands: [1, 3, 3, 3, 3, 0],
				points: [
					[0, bottom],
					[-6, 60 + frame.smile * 4],
					[-26, 50 - frame.smile * 7],
					[-24, corner],
					[-22, 31],
					[-6, 32],
					[0, 43],
					[6, 32],
					[22, 31],
					[24, corner],
					[26, 50 - frame.smile * 7],
					[6, 60 + frame.smile * 4],
					[0, bottom]
				]
			};
		}

		const halfWidth = 27 + Math.max(0, frame.smile) * 5;
		const cornerY = 51 - frame.smile * 7;
		const centerY = 51 + frame.smile * 9;
		const halfHeight = 2 + frame.mouthOpen * 15;

		return {
			commands: [1, 3, 3, 0],
			points: [
				[-halfWidth, cornerY],
				[-halfWidth * 0.55, centerY - halfHeight],
				[halfWidth * 0.55, centerY - halfHeight],
				[halfWidth, cornerY],
				[halfWidth * 0.55, centerY + halfHeight],
				[-halfWidth * 0.55, centerY + halfHeight],
				[-halfWidth, cornerY]
			]
		};
	}

	let displayPose = $derived.by(() => {
		if (!pose) return null;

		const distanceScale = clamp(1 + ((pose.scaleX + pose.scaleY) / 2 - 1) * 0.12, 0.9, 1.1);

		return {
			...pose,
			x: clamp(pose.x * 40, -28, 28),
			y: clamp(pose.y * 30, -22, 22),
			scaleX: distanceScale,
			scaleY: distanceScale,
			leftEyeOpen: clamp(pose.leftEyeOpen, 0.15, 1),
			rightEyeOpen: clamp(pose.rightEyeOpen, 0.15, 1),
			mouthOpen: clamp(1 - Math.pow(1 - pose.mouthOpen, 1.7), 0, 1),
			smile: clamp(pose.smile * 1.45, -1, 1)
		};
	});

	let paintTransforms = $derived.by((): Readonly<Record<string, Matrix>> => {
		const frame = displayPose ?? NEUTRAL_RIG_POSE;
		const radians = (frame.rotation * Math.PI) / 180;
		const cosine = Math.cos(radians);
		const sine = Math.sin(radians);
		const scaleX = frame.scaleX * (1 - Math.abs(frame.yaw) * 0.1);
		const scaleY = frame.scaleY;
		const faceX = frame.yaw * 10;
		const faceY = frame.pitch * 7;
		const eyeBaseX = parts.eyes === 'dots' ? 40 : parts.eyes === 'soft' ? 34 : 35;
		const eyeBaseY = parts.eyes === 'dots' ? -20 : parts.eyes === 'soft' ? -10 : -12;

		return {
			'rig-root': matrix(
				scaleX * cosine,
				-scaleY * sine,
				frame.x,
				scaleX * sine,
				scaleY * cosine,
				frame.y
			),
			'rig-head-plane': matrix(1, 0, faceX, 0, 1, faceY),
			'rig-eyes-plane': matrix(1, 0, faceX, 0, 1, faceY),
			'rig-nose-plane': matrix(1, 0, faceX, 0, 1, faceY),
			'rig-mouth-plane': matrix(1, 0, faceX, 0, 1, faceY),
			'rig-glasses-plane': matrix(1, 0, faceX, 0, 1, faceY),
			'rig-left-brow': matrix(1, 0, -eyeBaseX + frame.yaw * 3, 0, 1, -frame.leftBrowLift * 6),
			'rig-right-brow': matrix(1, 0, eyeBaseX + frame.yaw * 3, 0, 1, -frame.rightBrowLift * 6),
			'rig-left-eye': matrix(1, 0, -eyeBaseX, 0, frame.leftEyeOpen, eyeBaseY),
			'rig-right-eye': matrix(1, 0, eyeBaseX, 0, frame.rightEyeOpen, eyeBaseY),
			'rig-left-pupil': matrix(
				1,
				0,
				-eyeBaseX + frame.leftGazeX * 6 + frame.yaw * 2,
				0,
				1,
				eyeBaseY + frame.leftGazeY * 4
			),
			'rig-right-pupil': matrix(
				1,
				0,
				eyeBaseX + frame.rightGazeX * 6 + frame.yaw * 2,
				0,
				1,
				eyeBaseY + frame.rightGazeY * 4
			)
		};
	});

	let paintPaths = $derived({
		'rig-mouth-shape': animatedMouthPath(displayPose ?? NEUTRAL_RIG_POSE)
	});

	$effect(() => {
		const key = partsKey;
		let cancelled = false;

		void tick().then(() => {
			if (!cancelled && characterSvgElement) {
				characterAsset = { key, source: characterSvgElement.outerHTML };
			}
		});

		return () => {
			cancelled = true;
		};
	});
</script>

<ThorVGRenderer
	source={characterAsset.source}
	sourceKey={characterAsset.key}
	{paintTransforms}
	{paintPaths}
	{width}
	{height}
	{label}
	class={className}
	{onerror}
/>

<div class="pointer-events-none absolute size-0 overflow-hidden" aria-hidden="true">
	<ComposedAvatar
		pose={NEUTRAL_RIG_POSE}
		viewBox={characterViewBox}
		{parts}
		label="Character artwork source"
		bind:svgElement={characterSvgElement}
	/>
</div>
