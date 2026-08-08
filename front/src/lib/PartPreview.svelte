<script lang="ts">
	import type { PartKey } from '$lib/avatarParts';

	type Props = { part: PartKey; option: string };
	let { part, option }: Props = $props();
</script>

<svg class="h-full w-full" viewBox="0 0 72 56" aria-hidden="true">
	<ellipse cx="36" cy="29" rx="23" ry="24" fill="#111827" stroke="#374151" stroke-width="1.5" />
	{#if part === 'head'}
		{#if option === 'peach'}
			<path
				d="M36 7C51 7 59 16 58 30 57 44 49 51 36 53 23 51 15 44 14 30 13 16 21 7 36 7Z"
				fill="#f4b7a5"
				stroke="#fff5f2"
				stroke-width="2"
			/>
			<path d="M15 22Q23 6 36 12Q48 4 57 21Q46 15 38 22Q28 13 15 22Z" fill="#44377c" />
		{:else if option === 'cream'}
			<ellipse cx="36" cy="30" rx="22" ry="24" fill="#fff3dc" stroke="#643e45" stroke-width="2" />
			<path
				d="M16 19Q24 4 35 13Q41 3 48 14Q56 12 58 21Q47 17 42 24Q35 16 29 24Q23 17 16 19Z"
				fill="#f8c477"
			/>
		{:else if option === 'outline'}
			<circle cx="36" cy="29" r="23" fill="#fffdf7" stroke="#333" stroke-width="4" />
		{:else if option === 'lavender'}
			<rect
				x="13"
				y="7"
				width="46"
				height="46"
				rx="16"
				fill="#d8c7ff"
				stroke="#674ea7"
				stroke-width="3"
			/>
			<path d="M16 17Q36 3 56 17" fill="none" stroke="#9275d8" stroke-width="5" />
		{:else if option === 'mint'}
			<path
				d="M36 5C49 5 60 15 59 29 64 39 54 51 42 49 35 57 21 51 21 43 10 38 12 23 19 19 19 9 28 5 36 5Z"
				fill="#bdebd7"
				stroke="#397866"
				stroke-width="3"
			/>
		{:else if option === 'rose'}
			<ellipse
				cx="36"
				cy="30"
				rx="22"
				ry="24"
				fill="#f8c4cf"
				stroke="#8f4960"
				stroke-width="2"
			/><path d="M16 20Q26 4 35 14Q45 3 57 20" fill="#a94f76" />
		{:else if option === 'sky'}
			<ellipse
				cx="36"
				cy="30"
				rx="22"
				ry="24"
				fill="#bfe7ff"
				stroke="#3576a8"
				stroke-width="2"
			/><path d="M16 19Q27 4 36 14Q47 3 57 20" fill="none" stroke="#68b9eb" stroke-width="5" />
		{:else if option === 'cocoa'}
			<ellipse
				cx="36"
				cy="30"
				rx="22"
				ry="24"
				fill="#9b654f"
				stroke="#4b2d2a"
				stroke-width="2"
			/><path d="M15 20Q25 3 35 14Q46 2 58 20" fill="#3f2930" />
		{:else if option === 'robot'}
			<rect
				x="14"
				y="7"
				width="44"
				height="46"
				rx="10"
				fill="#a9bac8"
				stroke="#354653"
				stroke-width="3"
			/><circle cx="36" cy="4" r="3" fill="#ff6b8a" /><path
				d="M36 7V11"
				stroke="#354653"
				stroke-width="2"
			/>
		{:else}
			<ellipse
				cx="36"
				cy="30"
				rx="22"
				ry="24"
				fill="#ffd977"
				stroke="#9a661e"
				stroke-width="2"
			/><path d="M16 19Q27 3 36 14Q47 2 57 20" fill="none" stroke="#f39b3d" stroke-width="5" />
		{/if}
	{:else if part === 'eyes'}
		{#if option === 'sparkle'}
			<ellipse
				cx="26"
				cy="28"
				rx="9"
				ry="7"
				fill="white"
				stroke="#4b3f72"
				stroke-width="2"
			/><ellipse
				cx="46"
				cy="28"
				rx="9"
				ry="7"
				fill="white"
				stroke="#4b3f72"
				stroke-width="2"
			/><circle cx="27" cy="28" r="4" fill="#4b3f72" /><circle
				cx="47"
				cy="28"
				r="4"
				fill="#4b3f72"
			/><circle cx="28" cy="26" r="1.3" fill="#9ff3ff" /><circle
				cx="48"
				cy="26"
				r="1.3"
				fill="#9ff3ff"
			/>
		{:else if option === 'soft'}
			<path
				d="M17 29Q26 20 35 29M37 29Q46 20 55 29"
				fill="white"
				stroke="#89515a"
				stroke-width="3"
			/><circle cx="27" cy="27" r="3" fill="#643e45" /><circle
				cx="47"
				cy="27"
				r="3"
				fill="#643e45"
			/>
		{:else if option === 'dots'}
			<circle cx="25" cy="28" r="4" fill="#eee" /><circle cx="47" cy="28" r="4" fill="#eee" />
		{:else if option === 'stars'}
			<path
				d="M25 19l2.5 5 5.5.8-4 3.8.9 5.4-4.9-2.6-4.9 2.6.9-5.4-4-3.8 5.5-.8Z M47 19l2.5 5 5.5.8-4 3.8.9 5.4-4.9-2.6-4.9 2.6.9-5.4-4-3.8 5.5-.8Z"
				fill="#ffd56f"
				stroke="#7c5b1c"
				stroke-width="1.5"
			/>
		{:else if option === 'sleepy'}
			<path
				d="M17 29Q26 36 35 29M37 29Q46 36 55 29"
				fill="none"
				stroke="#f4c6d7"
				stroke-width="4"
				stroke-linecap="round"
			/>
		{:else if option === 'wide'}
			<ellipse
				cx="25"
				cy="28"
				rx="10"
				ry="8"
				fill="#fff"
				stroke="#315f79"
				stroke-width="2"
			/><ellipse
				cx="47"
				cy="28"
				rx="10"
				ry="8"
				fill="#fff"
				stroke="#315f79"
				stroke-width="2"
			/><circle cx="25" cy="28" r="4" fill="#28799a" /><circle
				cx="47"
				cy="28"
				r="4"
				fill="#28799a"
			/>
		{:else if option === 'wink'}
			<path d="M16 29Q25 37 34 29" fill="none" stroke="#6d415a" stroke-width="4" /><ellipse
				cx="47"
				cy="28"
				rx="9"
				ry="7"
				fill="#fff"
				stroke="#6d415a"
				stroke-width="2"
			/><circle cx="47" cy="28" r="3" fill="#6d415a" />
		{:else if option === 'hearts'}
			<path
				d="M25 36C20 31 16 28 16 23c0-6 7-7 9-2 2-5 9-4 9 2 0 5-4 8-9 13ZM47 36C42 31 38 28 38 23c0-6 7-7 9-2 2-5 9-4 9 2 0 5-4 8-9 13Z"
				fill="#ff668f"
			/>
		{:else if option === 'pixel'}
			<rect
				x="16"
				y="22"
				width="18"
				height="13"
				fill="#dff8ff"
				stroke="#27384a"
				stroke-width="2"
			/><rect
				x="38"
				y="22"
				width="18"
				height="13"
				fill="#dff8ff"
				stroke="#27384a"
				stroke-width="2"
			/><rect x="23" y="26" width="5" height="5" fill="#27384a" /><rect
				x="45"
				y="26"
				width="5"
				height="5"
				fill="#27384a"
			/>
		{:else}
			<ellipse
				cx="25"
				cy="28"
				rx="9"
				ry="7"
				fill="#fff"
				stroke="#593748"
				stroke-width="2"
			/><ellipse
				cx="47"
				cy="28"
				rx="9"
				ry="7"
				fill="#fff"
				stroke="#593748"
				stroke-width="2"
			/><circle cx="25" cy="28" r="3" fill="#593748" /><circle
				cx="47"
				cy="28"
				r="3"
				fill="#593748"
			/><path d="M18 22l-3-4M22 20l-1-5M54 22l3-4M50 20l1-5" stroke="#593748" stroke-width="2" />
		{/if}
	{:else if part === 'nose'}
		{#if option === 'curve'}<path
				d="M34 18Q40 30 33 35"
				fill="none"
				stroke="#cf817d"
				stroke-width="3"
				stroke-linecap="round"
			/>
		{:else if option === 'cat'}<path
				d="M29 27L36 32l7-5q-7-7-14 0Z"
				fill="#e98f9e"
				stroke="#643e45"
				stroke-width="2"
			/>
		{:else if option === 'button'}<circle
				cx="36"
				cy="29"
				r="7"
				fill="#ef9f9f"
				stroke="#733"
				stroke-width="2"
			/><circle cx="33.5" cy="27" r="1" fill="#733" /><circle cx="38.5" cy="27" r="1" fill="#733" />
		{:else if option === 'triangle'}<path
				d="M36 21l8 13H28Z"
				fill="#ffd56f"
				stroke="#7c5b1c"
				stroke-width="2"
			/>
		{:else if option === 'dot'}<circle cx="36" cy="29" r="3" fill="#5b3945" />
		{:else if option === 'oval'}<ellipse
				cx="36"
				cy="29"
				rx="5"
				ry="8"
				fill="#d99086"
				stroke="#78484b"
				stroke-width="2"
			/>
		{:else if option === 'diamond'}<path
				d="M36 21l7 8-7 8-7-8Z"
				fill="#84d9e8"
				stroke="#356776"
				stroke-width="2"
			/>
		{:else if option === 'pig'}<ellipse
				cx="36"
				cy="30"
				rx="10"
				ry="7"
				fill="#f29aaa"
				stroke="#8f4c5b"
				stroke-width="2"
			/><circle cx="33" cy="30" r="1.5" fill="#8f4c5b" /><circle
				cx="39"
				cy="30"
				r="1.5"
				fill="#8f4c5b"
			/>
		{:else if option === 'freckles'}<path
				d="M26 28h0m5 3h0m5-3h0m5 3h0m5-3h0"
				stroke="#a96758"
				stroke-width="3"
			/>
		{:else}<path d="M30 29h12" stroke="#6b7280" stroke-width="1.5" stroke-dasharray="2 3" />{/if}
	{:else if part === 'mouth'}
		{#if option === 'berry'}<path
				d="M22 29Q36 44 50 29Q36 23 22 29Z"
				fill="#8b315d"
				stroke="#f19ac2"
				stroke-width="2"
			/>
		{:else if option === 'rose'}<path
				d="M22 28Q36 42 50 28"
				fill="none"
				stroke="#e77f91"
				stroke-width="5"
				stroke-linecap="round"
			/>
		{:else if option === 'line'}<path
				d="M25 31h22"
				stroke="#eee"
				stroke-width="5"
				stroke-linecap="round"
			/>
		{:else if option === 'toothy'}<path
				d="M20 27Q36 43 52 27Q36 34 20 27Z"
				fill="white"
				stroke="#5c3650"
				stroke-width="2"
			/>
		{:else if option === 'heart'}<path
				d="M36 42C31 36 23 31 23 24c0-7 10-9 13-2 3-7 13-5 13 2 0 7-8 12-13 18Z"
				fill="#ff7895"
				stroke="#7e2940"
				stroke-width="2"
			/>
		{:else if option === 'coral'}<path
				d="M21 28Q36 44 51 28Q36 23 21 28Z"
				fill="#f47f73"
				stroke="#8e3f45"
				stroke-width="2"
			/>
		{:else if option === 'bubble'}<path
				d="M21 28Q36 44 51 28Q36 22 21 28Z"
				fill="#8768c9"
				stroke="#44306e"
				stroke-width="2"
			/><circle cx="42" cy="27" r="2" fill="#d9ccff" />
		{:else if option === 'vampire'}<path
				d="M20 27Q36 44 52 27Q36 22 20 27Z"
				fill="#4a2038"
				stroke="#24101c"
				stroke-width="2"
			/><path d="M26 28l4 9 4-9m4 0 4 9 4-9" fill="#fff" />
		{:else if option === 'gloss'}<path
				d="M21 28Q36 44 51 28Q36 22 21 28Z"
				fill="#ee5d9b"
				stroke="#8e2857"
				stroke-width="2"
			/><path d="M27 27q9-4 18 0" fill="none" stroke="#ffd7e9" stroke-width="2" />
		{:else}<ellipse
				cx="36"
				cy="30"
				rx="10"
				ry="13"
				fill="#31536f"
				stroke="#172d3d"
				stroke-width="3"
			/><ellipse cx="36" cy="32" rx="5" ry="3" fill="#9fdff0" opacity=".5" />{/if}
	{:else}
		{#if option === 'helmet'}<circle
				cx="36"
				cy="29"
				r="25"
				fill="#19162f99"
				stroke="#9ff3ff"
				stroke-width="3"
			/><path d="M45 7q6-8 11-2" fill="none" stroke="#9ff3ff" stroke-width="2" /><circle
				cx="58"
				cy="4"
				r="3"
				fill="#ff8fc7"
			/>
		{:else if option === 'cat-hood'}<path
				d="M13 25 17 3l15 12M59 25 55 3 40 15"
				fill="#e98f9e"
				stroke="#643e45"
				stroke-width="3"
			/><path d="M14 28Q14 5 36 7T58 28" fill="none" stroke="#e98f9e" stroke-width="8" />
		{:else if option === 'flowers'}<g fill="#ff8fc7" stroke="#fff" stroke-width="1"
				><circle cx="23" cy="11" r="6" /><circle cx="36" cy="7" r="6" /><circle
					cx="49"
					cy="11"
					r="6"
				/></g
			><g fill="#ffd56f"
				><circle cx="23" cy="11" r="2" /><circle cx="36" cy="7" r="2" /><circle
					cx="49"
					cy="11"
					r="2"
				/></g
			>
		{:else if option === 'glasses'}<circle
				cx="26"
				cy="28"
				r="10"
				fill="#9ff3ff22"
				stroke="#a7f3d0"
				stroke-width="3"
			/><circle cx="46" cy="28" r="10" fill="#9ff3ff22" stroke="#a7f3d0" stroke-width="3" /><path
				d="M36 28h0M16 25h-5M56 25h5"
				stroke="#a7f3d0"
				stroke-width="3"
			/>
		{:else if option === 'beanie'}<path
				d="M15 20Q17 2 36 2t21 18"
				fill="#4b72c2"
				stroke="#243d75"
				stroke-width="3"
			/><path
				d="M14 19q22-7 44 0v7q-22-6-44 0Z"
				fill="#7ea5ed"
				stroke="#243d75"
				stroke-width="2"
			/><circle cx="36" cy="2" r="4" fill="#ff89ad" />
		{:else if option === 'crown'}<path
				d="M19 18 17 2l10 8 9-12 9 12 10-8-2 16Z"
				fill="#ffd45f"
				stroke="#8a6518"
				stroke-width="2"
			/>
		{:else if option === 'bow'}<g transform="translate(18 14)"
				><path
					d="M0 0C-9-9-14 3-3 6ZM0 0C9-9 14 3 3 6Z"
					fill="#ff79a8"
					stroke="#873655"
					stroke-width="2"
				/><circle r="3" fill="#ffd36f" /></g
			>
		{:else if option === 'earrings'}<path
				d="M13 29v9m46-9v9"
				stroke="#e9b93f"
				stroke-width="2"
			/><circle cx="13" cy="43" r="5" fill="#7de0d5" stroke="#276d69" stroke-width="2" /><circle
				cx="59"
				cy="43"
				r="5"
				fill="#7de0d5"
				stroke="#276d69"
				stroke-width="2"
			/>
		{:else if option === 'monocle'}<circle
				cx="47"
				cy="28"
				r="11"
				fill="#f9e27a22"
				stroke="#d6b94c"
				stroke-width="3"
			/><path d="M56 34q7 10 4 17" fill="none" stroke="#d6b94c" stroke-width="2" />
		{:else}<path d="M24 29h24" stroke="#6b7280" stroke-width="1.5" stroke-dasharray="2 3" />{/if}
	{/if}
</svg>
