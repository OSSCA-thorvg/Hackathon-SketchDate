<script lang="ts">
	import type { PartSelection } from '$lib/avatarParts';
	import type { RigPose } from '$lib/characterRig';

	type Props = {
		pose: RigPose | null;
		viewBox: string;
		parts: PartSelection;
		label: string;
		svgElement?: SVGSVGElement;
	};

	let { pose, viewBox, parts, label, svgElement = $bindable() }: Props = $props();

	function rootTransform(frame: RigPose): string {
		const yawScale = 1 - Math.abs(frame.yaw) * 0.1;
		return `translate(${frame.x.toFixed(2)} ${frame.y.toFixed(2)}) rotate(${frame.rotation.toFixed(2)}) scale(${(frame.scaleX * yawScale).toFixed(4)} ${frame.scaleY.toFixed(4)})`;
	}

	function facePlaneTransform(frame: RigPose): string {
		return `translate(${(frame.yaw * 10).toFixed(2)} ${(frame.pitch * 7).toFixed(2)})`;
	}

	function browTransform(x: number, lift: number, yaw: number): string {
		return `translate(${(x + yaw * 3).toFixed(2)} ${(-lift * 6).toFixed(2)})`;
	}

	function pupilTransform(gazeX: number, gazeY: number, yaw: number): string {
		return `translate(${(gazeX * 6 + yaw * 2).toFixed(2)} ${(gazeY * 4).toFixed(2)})`;
	}

	function mouthPath(frame: RigPose): string {
		const halfWidth = 27 + Math.max(0, frame.smile) * 5;
		const cornerY = 51 - frame.smile * 7;
		const centerY = 51 + frame.smile * 9;
		const halfHeight = 2 + frame.mouthOpen * 15;
		return `M ${-halfWidth} ${cornerY} C ${-halfWidth * 0.55} ${centerY - halfHeight}, ${halfWidth * 0.55} ${centerY - halfHeight}, ${halfWidth} ${cornerY} C ${halfWidth * 0.55} ${centerY + halfHeight}, ${-halfWidth * 0.55} ${centerY + halfHeight}, ${-halfWidth} ${cornerY} Z`;
	}

	function smileCurvePath(frame: RigPose): string {
		const halfWidth = 27 + Math.max(0, frame.smile) * 5;
		const cornerY = 51 - frame.smile * 7;
		const centerY = 51 + frame.smile * 10;
		return `M ${-halfWidth} ${cornerY} Q 0 ${centerY} ${halfWidth} ${cornerY}`;
	}

	function starEyeScale(frame: RigPose | null, side: 'left' | 'right'): number {
		if (!frame) return 1;
		return Math.max(0.2, side === 'left' ? frame.leftEyeOpen : frame.rightEyeOpen);
	}
</script>

<svg
	bind:this={svgElement}
	xmlns="http://www.w3.org/2000/svg"
	class="pointer-events-none absolute inset-0 block h-full w-full -scale-x-100 object-contain"
	{viewBox}
	preserveAspectRatio="xMidYMid meet"
	aria-label={label}
>
	{#if pose}
		<title>{label}</title>
		<g id="rig-root" transform={rootTransform(pose)} stroke-linecap="round" stroke-linejoin="round">
			<!-- Back accessories -->
			{#if parts.accessory === 'helmet'}
				<path
					d="M-69 100 Q-88 118-91 145 H91 Q88 118 69 100Z"
					fill="#29234f"
					stroke="#9ff3ff"
					stroke-width="5"
				/>
				<path
					d="M-104-31 C-111-93-75-132 0-137 C75-132 111-93 104-31 L100 64 C87 112 53 126 0 128 C-53 126-87 112-100 64Z"
					fill="#19162f"
					stroke="#9ff3ff"
					stroke-width="6"
				/>
				<path
					d="M-91-54 Q-118-51-116-20 V29 Q-115 48-94 45"
					fill="#5147a5"
					stroke="#9ff3ff"
					stroke-width="5"
				/>
				<path
					d="M91-54 Q118-51 116-20 V29 Q115 48 94 45"
					fill="#5147a5"
					stroke="#9ff3ff"
					stroke-width="5"
				/>
				<circle cx="-115" cy="-4" r="8" fill="#ff8fc7" />
				<circle cx="115" cy="-4" r="8" fill="#ff8fc7" />
				<path d="M35-130 Q49-157 70-166" fill="none" stroke="#9ff3ff" stroke-width="5" />
				<circle cx="73" cy="-169" r="8" fill="#ff8fc7" stroke="#fff" stroke-width="3" />
			{:else if parts.accessory === 'cat-hood'}
				<path
					d="M-79-73 L-68-146 Q-65-158-54-150 L-16-113 M79-73 L68-146 Q65-158 54-150 L16-113"
					fill="#f49daa"
					stroke="#643e45"
					stroke-width="7"
				/>
				<path d="M-65-132 L-59-104 L-38-119Z M65-132 L59-104 L38-119Z" fill="#ffe0d6" />
				<path
					d="M0-125 C69-125 101-83 100-6 C100 64 77 117 46 137 Q22 126 0 140 Q-22 126-46 137 C-77 117-100 64-100-6 C-101-83-69-125 0-125Z"
					fill="#e98f9e"
					stroke="#643e45"
					stroke-width="7"
				/>
				<circle cx="-92" cy="6" r="12" fill="#ffd56f" stroke="#643e45" stroke-width="5" />
				<circle cx="92" cy="6" r="12" fill="#ffd56f" stroke="#643e45" stroke-width="5" />
			{:else if parts.accessory === 'flowers'}
				<path d="M-67-91 Q0-128 67-91" fill="none" stroke="#4f8b69" stroke-width="7" />
				{#each [-55, -28, 0, 28, 55] as flowerX (flowerX)}
					<g transform={`translate(${flowerX} ${-96 - Math.abs(flowerX) * 0.12})`}>
						<circle cx="-5" cy="0" r="7" fill="#ff8fc7" /><circle
							cx="5"
							cy="0"
							r="7"
							fill="#f9a8d4"
						/><circle cx="0" cy="-5" r="7" fill="#fbcfe8" /><circle
							cx="0"
							cy="5"
							r="7"
							fill="#ff78a5"
						/><circle r="3" fill="#ffd56f" />
					</g>
				{/each}
			{/if}

			<!-- Head/base -->
			<g id="rig-head-plane" transform={facePlaneTransform(pose)}>
				{#if parts.head === 'peach'}
					<path
						d="M0-104 C62-104 86-67 82-3 C79 63 51 101 0 108 C-51 101-79 63-82-3 C-86-67-62-104 0-104Z"
						fill="#f4b7a5"
						stroke="#fff5f2"
						stroke-width="5"
					/>
					<path
						d="M-77-36 Q-72-94-18-103 Q-35-75-5-62 Q20-91 70-70 Q79-53 77-28 Q44-55 16-39 Q-17-68-77-36Z"
						fill="#44377c"
					/>
					<path
						d="M-74-47 Q-24-76 12-48 Q45-69 74-42"
						fill="none"
						stroke="#6f5ec7"
						stroke-width="8"
					/>
					<circle cx="-58" cy="31" r="9" fill="#ef8e9c66" />
					<circle cx="58" cy="31" r="9" fill="#ef8e9c66" />
				{:else if parts.head === 'cream'}
					<path
						d="M0-99 C58-99 80-64 79-4 C78 64 54 101 0 109 C-54 101-78 64-79-4 C-80-64-58-99 0-99Z"
						fill="#fff3dc"
						stroke="#643e45"
						stroke-width="5"
					/>
					<path
						d="M-66-65 Q-42-105-5-92 Q8-113 25-90 Q53-94 68-63 Q43-71 24-52 Q3-75-14-52 Q-36-74-66-65Z"
						fill="#f8c477"
					/>
					<path
						d="M-51-80 Q-29-91-15-75 M15-75 Q29-91 51-80"
						fill="none"
						stroke="#fff7df"
						stroke-width="8"
					/>
					<circle cx="-56" cy="44" r="10" fill="#f3a2a788" />
					<circle cx="56" cy="44" r="10" fill="#f3a2a788" />
				{:else if parts.head === 'outline'}
					<!-- CC0 source shape: https://www.svgrepo.com/show/437856/face.svg -->
					<rect
						x="-91.4"
						y="-91.4"
						width="182.8"
						height="182.8"
						rx="91.4"
						fill="#fffdf7"
						stroke="#333333"
						stroke-width="12"
					/>
				{:else if parts.head === 'lavender'}
					<rect
						x="-82"
						y="-100"
						width="164"
						height="208"
						rx="55"
						fill="#d8c7ff"
						stroke="#674ea7"
						stroke-width="6"
					/>
					<path
						d="M-76-54 Q-38-112 0-91 Q39-117 76-53 Q44-70 14-48 Q-18-73-76-54Z"
						fill="#9275d8"
					/>
					<circle cx="-57" cy="40" r="10" fill="#ef9fca66" /><circle
						cx="57"
						cy="40"
						r="10"
						fill="#ef9fca66"
					/>
				{:else if parts.head === 'mint'}
					<path
						d="M0-105 C54-116 84-83 82-34 C105 9 77 54 61 82 C38 121-15 109-30 101 C-75 113-97 61-82 29 C-105-12-84-68-62-78 C-49-108-18-114 0-105Z"
						fill="#bdebd7"
						stroke="#397866"
						stroke-width="6"
					/>
					<path
						d="M-66-50 Q-35-100-3-82 Q25-109 67-56"
						fill="none"
						stroke="#6fb49a"
						stroke-width="12"
					/>
				{:else if parts.head === 'rose'}
					<path
						d="M0-103 C58-106 84-67 82-4 C80 65 50 105 0 110 C-50 105-80 65-82-4 C-84-67-58-106 0-103Z"
						fill="#f8c4cf"
						stroke="#8f4960"
						stroke-width="6"
					/>
					<path
						d="M-76-49 Q-51-108-8-89 Q16-117 72-55 Q42-66 15-45 Q-17-69-76-49Z"
						fill="#a94f76"
					/>
					<circle cx="-58" cy="38" r="10" fill="#ef799766" /><circle
						cx="58"
						cy="38"
						r="10"
						fill="#ef799766"
					/>
				{:else if parts.head === 'sky'}
					<ellipse
						cx="0"
						cy="2"
						rx="83"
						ry="108"
						fill="#bfe7ff"
						stroke="#3576a8"
						stroke-width="6"
					/>
					<path
						d="M-74-55 Q-42-108-5-82 Q24-111 73-54"
						fill="none"
						stroke="#68b9eb"
						stroke-width="13"
					/>
					<path
						d="M-64 55 Q-50 68-36 55 M36 55 Q50 68 64 55"
						fill="none"
						stroke="#79c9ee"
						stroke-width="5"
					/>
				{:else if parts.head === 'cocoa'}
					<path
						d="M0-105 C61-105 84-70 82-2 C80 68 52 106 0 110 C-52 106-80 68-82-2 C-84-70-61-105 0-105Z"
						fill="#9b654f"
						stroke="#4b2d2a"
						stroke-width="6"
					/>
					<path
						d="M-77-48 Q-57-105-13-94 Q7-118 72-57 Q42-69 15-45 Q-19-70-77-48Z"
						fill="#3f2930"
					/>
					<circle cx="-57" cy="39" r="9" fill="#d9827a55" /><circle
						cx="57"
						cy="39"
						r="9"
						fill="#d9827a55"
					/>
				{:else if parts.head === 'robot'}
					<rect
						x="-84"
						y="-99"
						width="168"
						height="205"
						rx="38"
						fill="#a9bac8"
						stroke="#354653"
						stroke-width="7"
					/>
					<path d="M-76-58 H76 M-70 66 H70" stroke="#718896" stroke-width="6" />
					<circle cx="0" cy="-113" r="9" fill="#ff6b8a" stroke="#354653" stroke-width="4" /><path
						d="M0-104 V-88"
						stroke="#354653"
						stroke-width="5"
					/>
					<circle cx="-68" cy="35" r="7" fill="#6de3c4" /><circle
						cx="68"
						cy="35"
						r="7"
						fill="#6de3c4"
					/>
				{:else}
					<path
						d="M0-106 C60-108 86-67 83-1 C80 67 52 104 0 110 C-52 104-80 67-83-1 C-86-67-60-108 0-106Z"
						fill="#ffd977"
						stroke="#9a661e"
						stroke-width="6"
					/>
					<path
						d="M-76-50 Q-53-104-7-90 Q17-115 73-55"
						fill="none"
						stroke="#f39b3d"
						stroke-width="13"
					/>
					<circle cx="-58" cy="39" r="10" fill="#f58f7c66" /><circle
						cx="58"
						cy="39"
						r="10"
						fill="#f58f7c66"
					/>
				{/if}
			</g>

			<!-- Eyes and brows -->
			<g id="rig-eyes-plane" transform={facePlaneTransform(pose)}>
				{#if parts.eyes === 'sparkle'}
					<g id="rig-left-brow" transform={browTransform(-35, pose.leftBrowLift, pose.yaw)}
						><path d="M-15-34 Q0-43 15-35" fill="none" stroke="#32294f" stroke-width="6" /></g
					>
					<g id="rig-right-brow" transform={browTransform(35, pose.rightBrowLift, pose.yaw)}
						><path d="M-15-35 Q0-43 15-34" fill="none" stroke="#32294f" stroke-width="6" /></g
					>
					<g id="rig-left-eye" transform={`translate(-35 -12) scale(1 ${pose.leftEyeOpen})`}>
						<ellipse rx="18" ry="12" fill="#fff" stroke="#32294f" stroke-width="4" />
					</g>
					<g id="rig-right-eye" transform={`translate(35 -12) scale(1 ${pose.rightEyeOpen})`}>
						<ellipse rx="18" ry="12" fill="#fff" stroke="#32294f" stroke-width="4" />
					</g>
					<g
						id="rig-left-pupil"
						transform={`translate(-35 -12) ${pupilTransform(pose.leftGazeX, pose.leftGazeY, pose.yaw)}`}
						><circle r={5 + pose.leftEyeOpen * 2} fill="#32294f" /><circle
							cx="2"
							cy="-2"
							r="1.7"
							fill="#9ff3ff"
						/></g
					>
					<g
						id="rig-right-pupil"
						transform={`translate(35 -12) ${pupilTransform(pose.rightGazeX, pose.rightGazeY, pose.yaw)}`}
						><circle r={5 + pose.rightEyeOpen * 2} fill="#32294f" /><circle
							cx="2"
							cy="-2"
							r="1.7"
							fill="#9ff3ff"
						/></g
					>
				{:else if parts.eyes === 'soft'}
					<g id="rig-left-brow" transform={browTransform(-34, pose.leftBrowLift, pose.yaw)}
						><path d="M-13-32 Q0-38 13-32" fill="none" stroke="#89515a" stroke-width="5" /></g
					>
					<g id="rig-right-brow" transform={browTransform(34, pose.rightBrowLift, pose.yaw)}
						><path d="M-13-32 Q0-38 13-32" fill="none" stroke="#89515a" stroke-width="5" /></g
					>
					<g id="rig-left-eye" transform={`translate(-34 -10) scale(1 ${pose.leftEyeOpen})`}>
						<ellipse rx="17" ry="11" fill="#fff" stroke="#643e45" stroke-width="4" />
					</g>
					<g id="rig-right-eye" transform={`translate(34 -10) scale(1 ${pose.rightEyeOpen})`}>
						<ellipse rx="17" ry="11" fill="#fff" stroke="#643e45" stroke-width="4" />
					</g>
					<g
						id="rig-left-pupil"
						transform={`translate(-34 -10) ${pupilTransform(pose.leftGazeX, pose.leftGazeY, pose.yaw)}`}
						><ellipse rx="5" ry={5 + pose.leftEyeOpen * 2} fill="#643e45" /><circle
							cx="1.5"
							cy="-2"
							r="1.5"
							fill="#fff"
						/></g
					>
					<g
						id="rig-right-pupil"
						transform={`translate(34 -10) ${pupilTransform(pose.rightGazeX, pose.rightGazeY, pose.yaw)}`}
						><ellipse rx="5" ry={5 + pose.rightEyeOpen * 2} fill="#643e45" /><circle
							cx="1.5"
							cy="-2"
							r="1.5"
							fill="#fff"
						/></g
					>
				{:else if parts.eyes === 'dots'}
					<g
						id="rig-left-eye"
						transform={`translate(-40 -20) ${pupilTransform(pose.leftGazeX, pose.leftGazeY, pose.yaw)}`}
						><ellipse rx="6" ry={Math.max(1.5, pose.leftEyeOpen * 6)} fill="#333333" /></g
					>
					<g
						id="rig-right-eye"
						transform={`translate(40 -20) ${pupilTransform(pose.rightGazeX, pose.rightGazeY, pose.yaw)}`}
						><ellipse rx="6" ry={Math.max(1.5, pose.rightEyeOpen * 6)} fill="#333333" /></g
					>
				{:else if parts.eyes === 'stars'}
					<g
						id="rig-left-eye"
						transform={`translate(-35 -12) scale(1 ${starEyeScale(pose, 'left')})`}
					>
						<path
							d="M0-14 4-5 14-4 7 3 9 13 0 8-9 13-7 3-14-4-4-5Z"
							fill="#ffd56f"
							stroke="#7c5b1c"
							stroke-width="3"
						/>
					</g>
					<g
						id="rig-right-eye"
						transform={`translate(35 -12) scale(1 ${starEyeScale(pose, 'right')})`}
					>
						<path
							d="M0-14 4-5 14-4 7 3 9 13 0 8-9 13-7 3-14-4-4-5Z"
							fill="#ffd56f"
							stroke="#7c5b1c"
							stroke-width="3"
						/>
					</g>
				{:else if parts.eyes === 'sleepy'}
					<path
						d="M-53-9 Q-35 6-17-9 M17-9 Q35 6 53-9"
						fill="none"
						stroke="#6d415a"
						stroke-width="7"
					/>
					<path
						d="M-50-1 l-7 5 M-43 2 l-4 7 M50-1 l7 5 M43 2 l4 7"
						stroke="#6d415a"
						stroke-width="3"
					/>
				{:else if parts.eyes === 'wide'}
					<g id="rig-left-eye" transform={`translate(-35 -12) scale(1 ${pose.leftEyeOpen})`}
						><ellipse rx="21" ry="15" fill="#fff" stroke="#315f79" stroke-width="4" /></g
					>
					<g id="rig-right-eye" transform={`translate(35 -12) scale(1 ${pose.rightEyeOpen})`}
						><ellipse rx="21" ry="15" fill="#fff" stroke="#315f79" stroke-width="4" /></g
					>
					<g
						id="rig-left-pupil"
						transform={`translate(-35 -12) ${pupilTransform(pose.leftGazeX, pose.leftGazeY, pose.yaw)}`}
						><circle r="8" fill="#28799a" /><circle cx="2" cy="-3" r="2" fill="#fff" /></g
					>
					<g
						id="rig-right-pupil"
						transform={`translate(35 -12) ${pupilTransform(pose.rightGazeX, pose.rightGazeY, pose.yaw)}`}
						><circle r="8" fill="#28799a" /><circle cx="2" cy="-3" r="2" fill="#fff" /></g
					>
				{:else if parts.eyes === 'wink'}
					<g id="rig-left-eye" transform={`translate(-35 -12) scale(1 ${pose.leftEyeOpen})`}
						><path d="M-18 2 Q0 15 18 2" fill="none" stroke="#6d415a" stroke-width="7" /></g
					>
					<g id="rig-right-eye" transform={`translate(35 -12) scale(1 ${pose.rightEyeOpen})`}
						><ellipse rx="18" ry="12" fill="#fff" stroke="#6d415a" stroke-width="4" /></g
					>
					<g
						id="rig-right-pupil"
						transform={`translate(35 -12) ${pupilTransform(pose.rightGazeX, pose.rightGazeY, pose.yaw)}`}
						><circle r="6" fill="#6d415a" /></g
					>
				{:else if parts.eyes === 'hearts'}
					<g id="rig-left-eye" transform={`translate(-35 -12) scale(1 ${pose.leftEyeOpen})`}
						><path
							d="M0 13 C-5 7-15 1-15-7 C-15-16-3-18 0-9 C3-18 15-16 15-7 C15 1 5 7 0 13Z"
							fill="#ff668f"
							stroke="#8f2948"
							stroke-width="3"
						/></g
					>
					<g id="rig-right-eye" transform={`translate(35 -12) scale(1 ${pose.rightEyeOpen})`}
						><path
							d="M0 13 C-5 7-15 1-15-7 C-15-16-3-18 0-9 C3-18 15-16 15-7 C15 1 5 7 0 13Z"
							fill="#ff668f"
							stroke="#8f2948"
							stroke-width="3"
						/></g
					>
				{:else if parts.eyes === 'pixel'}
					<g id="rig-left-eye" transform={`translate(-35 -12) scale(1 ${pose.leftEyeOpen})`}
						><rect
							x="-17"
							y="-11"
							width="34"
							height="22"
							fill="#dff8ff"
							stroke="#27384a"
							stroke-width="4"
						/></g
					>
					<g id="rig-right-eye" transform={`translate(35 -12) scale(1 ${pose.rightEyeOpen})`}
						><rect
							x="-17"
							y="-11"
							width="34"
							height="22"
							fill="#dff8ff"
							stroke="#27384a"
							stroke-width="4"
						/></g
					>
					<g
						id="rig-left-pupil"
						transform={`translate(-35 -12) ${pupilTransform(pose.leftGazeX, pose.leftGazeY, pose.yaw)}`}
						><rect x="-5" y="-5" width="10" height="10" fill="#27384a" /></g
					>
					<g
						id="rig-right-pupil"
						transform={`translate(35 -12) ${pupilTransform(pose.rightGazeX, pose.rightGazeY, pose.yaw)}`}
						><rect x="-5" y="-5" width="10" height="10" fill="#27384a" /></g
					>
				{:else}
					<g id="rig-left-eye" transform={`translate(-35 -12) scale(1 ${pose.leftEyeOpen})`}
						><ellipse rx="18" ry="11" fill="#fff" stroke="#593748" stroke-width="4" /><path
							d="M-15-8 L-20-15 M-8-12 L-10-20"
							stroke="#593748"
							stroke-width="4"
						/></g
					>
					<g id="rig-right-eye" transform={`translate(35 -12) scale(1 ${pose.rightEyeOpen})`}
						><ellipse rx="18" ry="11" fill="#fff" stroke="#593748" stroke-width="4" /><path
							d="M15-8 L20-15 M8-12 L10-20"
							stroke="#593748"
							stroke-width="4"
						/></g
					>
					<g
						id="rig-left-pupil"
						transform={`translate(-35 -12) ${pupilTransform(pose.leftGazeX, pose.leftGazeY, pose.yaw)}`}
						><circle r="6" fill="#593748" /></g
					>
					<g
						id="rig-right-pupil"
						transform={`translate(35 -12) ${pupilTransform(pose.rightGazeX, pose.rightGazeY, pose.yaw)}`}
						><circle r="6" fill="#593748" /></g
					>
				{/if}
			</g>

			<!-- Nose -->
			<g id="rig-nose-plane" transform={facePlaneTransform(pose)}>
				{#if parts.nose === 'curve'}
					<path d="M-5 2 Q4 20-5 25" fill="none" stroke="#cf817d" stroke-width="4" />
				{:else if parts.nose === 'cat'}
					<path
						d="M-7 15 L0 20 L7 15 Q0 9-7 15Z"
						fill="#e98f9e"
						stroke="#643e45"
						stroke-width="3"
					/>
					<path
						d="M0 20 V27 M-51 23 H-76 M-49 32 L-72 38 M51 23 H76 M49 32 L72 38"
						fill="none"
						stroke="#a86b6b"
						stroke-width="3"
						opacity=".72"
					/>
				{:else if parts.nose === 'button'}
					<circle cx="0" cy="19" r="10" fill="#ef9f9f" stroke="#733e4b" stroke-width="3" />
					<circle cx="-3.5" cy="17" r="1.5" fill="#733e4b" /><circle
						cx="3.5"
						cy="17"
						r="1.5"
						fill="#733e4b"
					/>
				{:else if parts.nose === 'triangle'}
					<path d="M0 8 L11 27 H-11Z" fill="#ffd56f" stroke="#7c5b1c" stroke-width="3" />
				{:else if parts.nose === 'dot'}
					<circle cx="0" cy="19" r="5" fill="#5b3945" />
				{:else if parts.nose === 'oval'}
					<ellipse cx="0" cy="18" rx="8" ry="12" fill="#d99086" stroke="#78484b" stroke-width="3" />
				{:else if parts.nose === 'diamond'}
					<path d="M0 7 L10 19 L0 30 L-10 19Z" fill="#84d9e8" stroke="#356776" stroke-width="3" />
				{:else if parts.nose === 'pig'}
					<ellipse
						cx="0"
						cy="20"
						rx="15"
						ry="11"
						fill="#f29aaa"
						stroke="#8f4c5b"
						stroke-width="3"
					/>
					<ellipse cx="-5" cy="20" rx="2.5" ry="4" fill="#8f4c5b" /><ellipse
						cx="5"
						cy="20"
						rx="2.5"
						ry="4"
						fill="#8f4c5b"
					/>
				{:else if parts.nose === 'freckles'}
					<path
						d="M-20 18 h0 M-11 22 h0 M0 18 h0 M11 22 h0 M20 18 h0"
						stroke="#a96758"
						stroke-width="6"
					/>
				{/if}
			</g>

			<!-- Mouth -->
			<g id="rig-mouth-plane" transform={facePlaneTransform(pose)}>
				<g id="rig-mouth">
					{#if parts.mouth === 'berry'}
						{#if pose.mouthOpen > 0.12}
							<path
								id="rig-mouth-shape"
								d={mouthPath(pose)}
								fill="#8b315d"
								stroke="#32294f"
								stroke-width="4"
							/>
						{:else}
							<path d={smileCurvePath(pose)} fill="none" stroke="#8b315d" stroke-width="7" />
						{/if}
						{#if pose.mouthOpen > 0.24}<path
								d="M-15 54 Q0 64 15 54 Q11 69 0 72 Q-11 69-15 54Z"
								fill="#ff8fc7"
								opacity={pose.mouthOpen}
							/>{/if}
					{:else if parts.mouth === 'rose'}
						{#if pose.mouthOpen > 0.12}
							<path
								id="rig-mouth-shape"
								d={mouthPath(pose)}
								fill="#b85565"
								stroke="#643e45"
								stroke-width="4"
							/>
						{:else}
							<path d={smileCurvePath(pose)} fill="none" stroke="#b85565" stroke-width="7" />
						{/if}
						{#if pose.mouthOpen > 0.24}<ellipse
								cx="0"
								cy="59"
								rx="12"
								ry={pose.mouthOpen * 7}
								fill="#ffb5b8"
							/>{/if}
					{:else if parts.mouth === 'line' && pose.mouthOpen > 0.08}
						<path
							id="rig-mouth-shape"
							d={mouthPath(pose)}
							fill="#333333"
							stroke="#333333"
							stroke-width="6"
						/>
					{:else if parts.mouth === 'line'}
						<path
							d={`M -18 ${48 - pose.smile * 5} Q 0 ${48 + pose.smile * 8} 18 ${48 - pose.smile * 5}`}
							fill="none"
							stroke="#333333"
							stroke-width="12"
						/>
					{:else if parts.mouth === 'toothy'}
						<path
							id="rig-mouth-shape"
							d={mouthPath(pose)}
							fill="#5c3650"
							stroke="#4a263d"
							stroke-width="4"
						/>
						<path
							d={`M -23 ${48 - pose.smile * 5} Q 0 ${55 + pose.smile * 6} 23 ${48 - pose.smile * 5} Q 0 ${52 + pose.smile * 4} -23 ${48 - pose.smile * 5}Z`}
							fill="white"
						/>
					{:else if parts.mouth === 'heart'}
						<path
							id="rig-mouth-shape"
							d={`M 0 ${68 + pose.mouthOpen * 5} C -6 ${60 + pose.smile * 4}, -26 ${50 - pose.smile * 7}, -24 ${41 - pose.smile * 5} C -22 31, -6 32, 0 43 C 6 32, 22 31, 24 ${41 - pose.smile * 5} C 26 ${50 - pose.smile * 7}, 6 ${60 + pose.smile * 4}, 0 ${68 + pose.mouthOpen * 5}Z`}
							fill="#ff7895"
							stroke="#7e2940"
							stroke-width="4"
						/>
					{:else if parts.mouth === 'coral'}
						<path
							id="rig-mouth-shape"
							d={mouthPath(pose)}
							fill="#f47f73"
							stroke="#8e3f45"
							stroke-width="4"
						/>
					{:else if parts.mouth === 'bubble'}
						<path
							id="rig-mouth-shape"
							d={mouthPath(pose)}
							fill="#8768c9"
							stroke="#44306e"
							stroke-width="4"
						/>
						<circle cx="10" cy="49" r="3" fill="#d9ccff" opacity=".8" />
					{:else if parts.mouth === 'vampire'}
						<path
							id="rig-mouth-shape"
							d={mouthPath(pose)}
							fill="#4a2038"
							stroke="#24101c"
							stroke-width="4"
						/>
						<path d="M-16 48 L-9 63 L-3 49 M3 49 L9 63 L16 48" fill="#fffdf2" />
					{:else if parts.mouth === 'gloss'}
						<path
							id="rig-mouth-shape"
							d={mouthPath(pose)}
							fill="#ee5d9b"
							stroke="#8e2857"
							stroke-width="4"
						/>
						<path
							d="M-15 48 Q0 43 15 48"
							fill="none"
							stroke="#ffd7e9"
							stroke-width="3"
							opacity=".9"
						/>
					{:else}
						<path
							id="rig-mouth-shape"
							d={mouthPath(pose)}
							fill="#31536f"
							stroke="#172d3d"
							stroke-width="5"
						/>
						<ellipse cx="0" cy="54" rx="10" ry="5" fill="#9fdff0" opacity=".45" />
					{/if}
				</g>
			</g>

			<!-- Front accessory accents -->
			{#if parts.accessory === 'helmet'}
				<path
					d="M-78-88 Q-49-116 0-118 Q49-116 78-88"
					fill="none"
					stroke="#7368df"
					stroke-width="12"
				/>
				<path d="M-89-71 Q-105-9-86 62" fill="none" stroke="#ffffff44" stroke-width="5" />
				<path d="M82-81 Q101-55 99-17" fill="none" stroke="#9ff3ff66" stroke-width="4" />
				<path d="M-67 118 Q0 137 67 118" fill="none" stroke="#7368df" stroke-width="9" />
			{:else if parts.accessory === 'cat-hood'}
				<path d="M-54 113 Q0 142 54 113" fill="#ffd56f" stroke="#643e45" stroke-width="6" />
				<path d="M-83-69 Q-98-9-82 55" fill="none" stroke="#ffd1d8" stroke-width="6" />
				<path d="M80-82 Q96-55 96-21" fill="none" stroke="#ffffff55" stroke-width="5" />
			{:else if parts.accessory === 'glasses'}
				<g
					id="rig-glasses-plane"
					transform={facePlaneTransform(pose)}
					fill="#9ff3ff1f"
					stroke="#a7f3d0"
					stroke-width="5"
				>
					<circle cx="-36" cy="-12" r="25" /><circle cx="36" cy="-12" r="25" />
					<path d="M-11-12 Q0-19 11-12 M-61-15 H-82 M61-15 H82" fill="none" />
				</g>
			{:else if parts.accessory === 'beanie'}
				<path
					d="M-78-69 Q-74-139 0-145 Q74-139 78-69Z"
					fill="#4b72c2"
					stroke="#243d75"
					stroke-width="6"
				/>
				<path
					d="M-82-72 Q0-91 82-72 V-53 Q0-72-82-53Z"
					fill="#7ea5ed"
					stroke="#243d75"
					stroke-width="6"
				/>
				<circle cx="0" cy="-151" r="12" fill="#ff89ad" stroke="#7c3150" stroke-width="4" />
			{:else if parts.accessory === 'crown'}
				<path
					d="M-58-91 L-66-148 L-28-121 L0-158 L28-121 L66-148 L58-91Z"
					fill="#ffd45f"
					stroke="#8a6518"
					stroke-width="6"
				/>
				<path d="M-59-95 H59" stroke="#fff1a8" stroke-width="9" /><circle
					cx="0"
					cy="-126"
					r="7"
					fill="#ff668f"
				/>
			{:else if parts.accessory === 'bow'}
				<g transform="translate(-80 -79)"
					><path
						d="M0 0 C-28-29-40 7-8 14Z M0 0 C28-29 40 7 8 14Z"
						fill="#ff79a8"
						stroke="#873655"
						stroke-width="5"
					/><circle r="10" fill="#ffd36f" stroke="#873655" stroke-width="4" /></g
				>
			{:else if parts.accessory === 'earrings'}
				<path d="M-87 15 V38 M87 15 V38" stroke="#e9b93f" stroke-width="5" /><circle
					cx="-87"
					cy="49"
					r="11"
					fill="#7de0d5"
					stroke="#276d69"
					stroke-width="4"
				/><circle cx="87" cy="49" r="11" fill="#7de0d5" stroke="#276d69" stroke-width="4" />
			{:else if parts.accessory === 'monocle'}
				<g
					id="rig-glasses-plane"
					transform={facePlaneTransform(pose)}
					fill="#f9e27a22"
					stroke="#d6b94c"
					stroke-width="5"
				>
					<circle cx="35" cy="-12" r="27" /><path d="M60 1 Q79 35 69 73" fill="none" /><circle
						cx="69"
						cy="78"
						r="5"
						fill="#d6b94c"
					/>
				</g>
			{/if}
		</g>
	{/if}
</svg>
