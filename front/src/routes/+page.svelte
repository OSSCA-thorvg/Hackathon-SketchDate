<script lang="ts">
	import CharacterPicker from '$lib/CharacterPicker.svelte';
	import CharacterRenderer from '$lib/CharacterRenderer.svelte';
	import { DEFAULT_PARTS, type PartSelection } from '$lib/avatarParts';
	import { NEUTRAL_RIG_POSE, type RigPose } from '$lib/characterRig';
	import { createRigPose, smoothRigPose } from '$lib/faceRig';
	import { onMount } from 'svelte';

	type AppState =
		'landing' | 'media-ready' | 'matching' | 'avatar-date' | 'decision' | 'video-date' | 'ended';
	type FaceLandmarkerInstance = import('@mediapipe/tasks-vision').FaceLandmarker;
	type ServerEnvelope = {
		v: number;
		type: string;
		roomId?: string;
		payload?: Record<string, unknown>;
	};

	const MEDIAPIPE_VERSION = '1.0.1';
	const WASM_URL = `https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@${MEDIAPIPE_VERSION}/wasm`;
	const MODEL_URL =
		'https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task';
	const BACKEND_WS_URL = 'wss://dlrutfjsooho5.cloudfront.net/ws';

	let appState = $state<AppState>('landing');
	let selectedParts = $state<PartSelection>({ ...DEFAULT_PARTS });
	let remoteParts = $state<PartSelection>({ ...DEFAULT_PARTS });
	let localPose = $state<RigPose | null>(null);
	let remotePose = $state<RigPose | null>(null);
	let now = $state(Date.now());
	let preparing = $state(false);
	let modelReady = $state(false);
	let cameraReady = $state(false);
	let microphoneReady = $state(false);
	let faceDetected = $state(false);
	let muted = $state(false);
	let connectionStatus = $state('Connecting securely');
	let choice = $state<'reveal' | 'end' | null>(null);
	let videoReady = $state(false);
	let endedReason = $state('This date has ended.');
	let isError = $state(false);
	let roundEndsAt = $state(0);
	let decisionEndsAt = $state(0);

	let trackerVideo = $state<HTMLVideoElement>(undefined!);
	let remoteAudio = $state<HTMLAudioElement>(undefined!);
	let remoteVideo = $state<HTMLVideoElement>(undefined!);
	let selfVideo = $state<HTMLVideoElement>(undefined!);
	let setupStageWidth = $state(0);
	let setupStageHeight = $state(0);
	let mediaStream: MediaStream | null = null;
	let remoteVideoStream: MediaStream | null = null;
	let faceLandmarker: FaceLandmarkerInstance | null = null;
	let modelPromise: Promise<void> | null = null;
	let animationFrameId: number | null = null;
	let lastVideoTime = -1;
	let lastMotionSentAt = 0;
	let smoothedPose: RigPose | null = null;

	let socket: WebSocket | null = null;
	let peerConnection: RTCPeerConnection | null = null;
	let videoTransceiver: RTCRtpTransceiver | null = null;
	let avatarChannel: RTCDataChannel | null = null;
	let motionChannel: RTCDataChannel | null = null;
	let roomId: string | null = null;
	let isInitiator = false;
	let iceServers: RTCIceServer[] | null = null;
	let pendingSignals: ServerEnvelope[] = [];
	let pendingCandidates: RTCIceCandidateInit[] = [];
	let intentionalSocketClose = false;

	let countdown = $derived(
		Math.max(0, Math.ceil(((appState === 'decision' ? decisionEndsAt : roundEndsAt) - now) / 1000))
	);
	let readyToMatch = $derived(modelReady && cameraReady && microphoneReady && faceDetected);

	onMount(() => {
		const clock = window.setInterval(() => (now = Date.now()), 250);
		return () => {
			window.clearInterval(clock);
			cleanupNetwork();
			stopMedia();
			faceLandmarker?.close();
			faceLandmarker = null;
		};
	});

	$effect(() => {
		const parts = selectedParts;
		if (avatarChannel?.readyState === 'open') {
			avatarChannel.send(JSON.stringify(parts));
		}
	});

	$effect(() => {
		if (remoteVideo && remoteVideoStream) {
			remoteVideo.srcObject = remoteVideoStream;
			void remoteVideo.play().catch(() => {});
		}
		if (selfVideo && mediaStream && appState === 'video-date') {
			selfVideo.srcObject = mediaStream;
			void selfVideo.play().catch(() => {});
		}
	});

	async function prepareMedia(): Promise<void> {
		appState = 'media-ready';
		preparing = true;
		isError = false;
		try {
			if (!navigator.mediaDevices?.getUserMedia) {
				throw new Error('Camera and microphone access are not supported in this browser.');
			}

			const modelReadyPromise = initializeFaceModel();
			const stream = await navigator.mediaDevices.getUserMedia({
				audio: { echoCancellation: true, noiseSuppression: true },
				video: { facingMode: 'user', width: { ideal: 1280 }, height: { ideal: 720 } }
			});
			mediaStream = stream;
			await modelReadyPromise;
			cameraReady = stream.getVideoTracks().some((track) => track.readyState === 'live');
			microphoneReady = stream.getAudioTracks().some((track) => track.readyState === 'live');
			trackerVideo.srcObject = stream;
			await waitForVideoMetadata(trackerVideo);
			await trackerVideo.play();
			startFaceTracking();
		} catch (error) {
			fail(getMediaErrorMessage(error));
		} finally {
			preparing = false;
		}
	}

	async function initializeFaceModel(): Promise<void> {
		if (faceLandmarker) return;
		if (modelPromise) return modelPromise;

		modelPromise = (async () => {
			const { FaceLandmarker, FilesetResolver } = await import('@mediapipe/tasks-vision');
			const vision = await FilesetResolver.forVisionTasks(WASM_URL);
			faceLandmarker = await FaceLandmarker.createFromOptions(vision, {
				baseOptions: { modelAssetPath: MODEL_URL },
				runningMode: 'VIDEO',
				numFaces: 1,
				minFaceDetectionConfidence: 0.5,
				minFacePresenceConfidence: 0.5,
				minTrackingConfidence: 0.5,
				outputFaceBlendshapes: true,
				outputFacialTransformationMatrixes: false
			});
			modelReady = true;
		})();

		try {
			await modelPromise;
		} finally {
			modelPromise = null;
		}
	}

	function startFaceTracking(): void {
		if (animationFrameId !== null) cancelAnimationFrame(animationFrameId);
		lastVideoTime = -1;
		animationFrameId = requestAnimationFrame(trackFace);
	}

	function trackFace(timestamp: number): void {
		if (!faceLandmarker || !mediaStream || !trackerVideo) return;

		if (
			trackerVideo.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA &&
			trackerVideo.currentTime !== lastVideoTime
		) {
			lastVideoTime = trackerVideo.currentTime;
			const result = faceLandmarker.detectForVideo(trackerVideo, performance.now());
			const landmarks = result.faceLandmarks[0];

			if (landmarks) {
				const target = createRigPose(
					landmarks,
					trackerVideo.videoWidth,
					trackerVideo.videoHeight,
					result.faceBlendshapes[0]?.categories ?? []
				);
				smoothedPose = smoothedPose ? smoothRigPose(smoothedPose, target) : target;
				localPose = smoothedPose;
				faceDetected = true;
			} else {
				localPose = null;
				smoothedPose = null;
				faceDetected = false;
			}

			if (timestamp - lastMotionSentAt >= 50 && motionChannel?.readyState === 'open') {
				motionChannel.send(JSON.stringify(localPose ?? NEUTRAL_RIG_POSE));
				lastMotionSentAt = timestamp;
			}
		}

		animationFrameId = requestAnimationFrame(trackFace);
	}

	async function startMatching(): Promise<void> {
		if (!readyToMatch || socket) return;
		appState = 'matching';
		connectionStatus = 'Looking for someone new';
		intentionalSocketClose = false;

		try {
			const nextSocket = new WebSocket(BACKEND_WS_URL);
			socket = nextSocket;
			nextSocket.onmessage = (event) => void handleServerMessage(event.data);
			nextSocket.onclose = () => {
				if (!intentionalSocketClose && appState !== 'ended') {
					fail('The secure connection closed. Please start again.');
				}
			};
			nextSocket.onerror = () => {
				if (appState === 'matching') connectionStatus = 'Still trying to reach the server';
			};

			await new Promise<void>((resolve, reject) => {
				const timer = window.setTimeout(
					() => reject(new Error('The server did not respond.')),
					10000
				);
				nextSocket.addEventListener(
					'open',
					() => {
						window.clearTimeout(timer);
						resolve();
					},
					{ once: true }
				);
				nextSocket.addEventListener(
					'close',
					() => {
						window.clearTimeout(timer);
						reject(new Error('Could not reach the matching server.'));
					},
					{ once: true }
				);
			});
			sendServer('match.join');
		} catch (error) {
			fail(error instanceof Error ? error.message : String(error));
		}
	}

	async function handleServerMessage(raw: string): Promise<void> {
		try {
			const message = JSON.parse(raw) as ServerEnvelope;
			if (message.v !== 1) return;

			switch (message.type) {
				case 'match.queued':
					connectionStatus = 'Looking for someone new';
					break;
				case 'match.found':
					roomId = message.roomId ?? null;
					isInitiator = message.payload?.initiator === true;
					appState = 'avatar-date';
					connectionStatus = 'Creating a private room';
					await tryStartPeer();
					break;
				case 'turn.credentials': {
					const supplied = message.payload?.iceServers;
					iceServers = Array.isArray(supplied)
						? (supplied as RTCIceServer[])
						: ((message.payload as unknown as RTCIceServer[]) ?? []);
					await tryStartPeer();
					break;
				}
				case 'signal.offer':
				case 'signal.answer':
				case 'signal.ice':
					if (peerConnection) await acceptSignal(message);
					else pendingSignals.push(message);
					break;
				case 'round.started':
					roundEndsAt = Number(message.payload?.endsAt ?? Date.now() + 180000);
					break;
				case 'decision.open':
					appState = 'decision';
					choice = null;
					decisionEndsAt = Number(message.payload?.endsAt ?? Date.now() + 20000);
					break;
				case 'reveal.granted':
					appState = 'video-date';
					videoReady = false;
					await enableVideoReveal();
					break;
				case 'session.ended':
					endLocally(reasonLabel(String(message.payload?.reason ?? 'ended')));
					break;
				case 'server.error':
					fail(String(message.payload?.message ?? 'The server could not continue this date.'));
					break;
			}
		} catch (error) {
			fail(error instanceof Error ? error.message : String(error));
		}
	}

	async function tryStartPeer(): Promise<void> {
		if (peerConnection || !roomId || !iceServers) return;

		const pc = new RTCPeerConnection({ iceServers, iceTransportPolicy: 'relay' });
		peerConnection = pc;
		remoteVideoStream = new MediaStream();
		videoTransceiver = pc.addTransceiver('video', { direction: 'recvonly' });

		for (const track of mediaStream?.getAudioTracks() ?? []) {
			pc.addTrack(track, mediaStream!);
		}

		pc.onicecandidate = (event) => {
			if (event.candidate) sendServer('signal.ice', event.candidate.toJSON());
		};
		pc.onconnectionstatechange = () => {
			if (pc.connectionState === 'connected') connectionStatus = 'Live and private';
			if (pc.connectionState === 'failed') fail('The private connection could not be established.');
		};
		pc.ontrack = (event) => {
			if (event.track.kind === 'audio') {
				remoteAudio.srcObject = new MediaStream([event.track]);
				void remoteAudio.play().catch(() => {});
			} else {
				remoteVideoStream?.addTrack(event.track);
				if (remoteVideo) {
					remoteVideo.srcObject = remoteVideoStream;
					void remoteVideo.play().catch(() => {});
				}
			}
		};
		pc.ondatachannel = (event) => connectDataChannel(event.channel);

		if (isInitiator) {
			connectDataChannel(pc.createDataChannel('avatar', { ordered: true }));
			connectDataChannel(pc.createDataChannel('motion', { ordered: false, maxRetransmits: 0 }));
			await sendOffer();
		}

		const signals = pendingSignals;
		pendingSignals = [];
		for (const signal of signals) await acceptSignal(signal);
	}

	function connectDataChannel(channel: RTCDataChannel): void {
		if (channel.label === 'avatar') {
			avatarChannel = channel;
			channel.onopen = () => channel.send(JSON.stringify(selectedParts));
			channel.onmessage = (event) => {
				const candidate = JSON.parse(String(event.data)) as PartSelection;
				if (isPartSelection(candidate)) remoteParts = candidate;
			};
		} else if (channel.label === 'motion') {
			motionChannel = channel;
			channel.onmessage = (event) => {
				const candidate = JSON.parse(String(event.data)) as RigPose;
				if (isRigPose(candidate)) remotePose = candidate;
			};
		}
	}

	async function acceptSignal(message: ServerEnvelope): Promise<void> {
		const pc = peerConnection;
		if (!pc || !message.payload) return;

		if (message.type === 'signal.ice') {
			const candidate = message.payload as RTCIceCandidateInit;
			if (pc.remoteDescription) await pc.addIceCandidate(candidate);
			else pendingCandidates.push(candidate);
			return;
		}

		if (message.type === 'signal.offer') {
			await pc.setRemoteDescription(message.payload as unknown as RTCSessionDescriptionInit);
			await flushCandidates();
			await pc.setLocalDescription(await pc.createAnswer());
			sendServer('signal.answer', pc.localDescription?.toJSON());
		} else if (message.type === 'signal.answer') {
			await pc.setRemoteDescription(message.payload as unknown as RTCSessionDescriptionInit);
			await flushCandidates();
		}
	}

	async function flushCandidates(): Promise<void> {
		const pc = peerConnection;
		if (!pc?.remoteDescription) return;
		const candidates = pendingCandidates;
		pendingCandidates = [];
		for (const candidate of candidates) await pc.addIceCandidate(candidate);
	}

	async function sendOffer(): Promise<void> {
		if (!peerConnection) return;
		await peerConnection.setLocalDescription(await peerConnection.createOffer());
		sendServer('signal.offer', peerConnection.localDescription?.toJSON());
	}

	async function enableVideoReveal(): Promise<void> {
		const cameraTrack = mediaStream?.getVideoTracks()[0];
		if (!peerConnection || !videoTransceiver || !cameraTrack) {
			fail('The camera is no longer available for video reveal.');
			return;
		}
		await videoTransceiver.sender.replaceTrack(cameraTrack);
		videoTransceiver.direction = 'sendrecv';
		if (isInitiator) await sendOffer();
	}

	function submitDecision(nextChoice: 'reveal' | 'end'): void {
		if (choice) return;
		choice = nextChoice;
		sendServer('decision.submit', { decision: nextChoice });
	}

	function toggleMute(): void {
		muted = !muted;
		for (const track of mediaStream?.getAudioTracks() ?? []) track.enabled = !muted;
	}

	function cancelMatching(): void {
		sendServer('match.cancel');
		cleanupNetwork();
		appState = 'media-ready';
	}

	function leaveDate(): void {
		sendServer('session.leave');
		endLocally('You ended the date.');
	}

	function endLocally(reason: string): void {
		endedReason = reason;
		isError = false;
		cleanupNetwork();
		stopMedia();
		appState = 'ended';
	}

	function fail(message: string): void {
		endedReason = message;
		isError = true;
		cleanupNetwork();
		stopMedia();
		appState = 'ended';
	}

	function startOver(): void {
		cleanupNetwork();
		stopMedia();
		selectedParts = { ...DEFAULT_PARTS };
		remoteParts = { ...DEFAULT_PARTS };
		remotePose = null;
		choice = null;
		videoReady = false;
		endedReason = 'This date has ended.';
		isError = false;
		appState = 'landing';
	}

	function beginSetup(): void {
		void prepareMedia();
	}

	function cleanupNetwork(): void {
		intentionalSocketClose = true;
		avatarChannel?.close();
		motionChannel?.close();
		peerConnection?.close();
		if (socket?.readyState === WebSocket.OPEN) socket.close(1000);
		else socket?.close();
		avatarChannel = null;
		motionChannel = null;
		peerConnection = null;
		videoTransceiver = null;
		socket = null;
		roomId = null;
		iceServers = null;
		pendingSignals = [];
		pendingCandidates = [];
		remoteVideoStream = null;
		if (remoteAudio) remoteAudio.srcObject = null;
	}

	function stopMedia(): void {
		if (animationFrameId !== null) cancelAnimationFrame(animationFrameId);
		animationFrameId = null;
		for (const track of mediaStream?.getTracks() ?? []) track.stop();
		mediaStream = null;
		if (trackerVideo) trackerVideo.srcObject = null;
		if (selfVideo) selfVideo.srcObject = null;
		localPose = null;
		smoothedPose = null;
		cameraReady = false;
		microphoneReady = false;
		faceDetected = false;
		muted = false;
	}

	function sendServer(type: string, payload?: unknown): void {
		if (socket?.readyState !== WebSocket.OPEN) return;
		socket.send(
			JSON.stringify({ v: 1, type, ...(roomId ? { roomId } : {}), ...(payload ? { payload } : {}) })
		);
	}

	function waitForVideoMetadata(video: HTMLVideoElement): Promise<void> {
		if (video.readyState >= HTMLMediaElement.HAVE_METADATA) return Promise.resolve();
		return new Promise((resolve, reject) => {
			video.addEventListener('loadedmetadata', () => resolve(), { once: true });
			video.addEventListener(
				'error',
				() => reject(new Error('Could not read the camera stream.')),
				{
					once: true
				}
			);
		});
	}

	function getMediaErrorMessage(error: unknown): string {
		if (error instanceof DOMException && error.name === 'NotAllowedError') {
			return 'Camera or microphone permission was denied. Both are needed for a date.';
		}
		if (error instanceof DOMException && error.name === 'NotFoundError') {
			return 'A working camera and microphone could not be found.';
		}
		return error instanceof Error ? error.message : String(error);
	}

	function reasonLabel(reason: string): string {
		const labels: Record<string, string> = {
			left: 'Your date ended the conversation.',
			peer_disconnected: 'Your date disconnected.',
			decision_declined: 'This date ended privately.',
			decision_timeout: 'The private decision time ran out.'
		};
		return labels[reason] ?? 'This date has ended.';
	}

	function formatTime(seconds: number): string {
		return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
	}

	function isPartSelection(value: PartSelection): boolean {
		return Boolean(value?.head && value?.eyes && value?.nose && value?.mouth && value?.accessory);
	}

	function isRigPose(value: RigPose): boolean {
		return typeof value?.x === 'number' && typeof value?.mouthOpen === 'number';
	}
</script>

<svelte:head>
	<title>SketchDate — Meet face to face, at your pace</title>
	<meta
		name="description"
		content="A private one-to-one voice date where your expressions animate a character until you both choose to meet face to face."
	/>
	<meta property="og:type" content="website" />
	<meta property="og:title" content="SketchDate — A little mystery. A real connection." />
	<meta
		property="og:description"
		content="Meet through voice and expression. Your camera stays private until you both choose to be seen."
	/>
	<meta property="og:image" content="https://ossca-thorvg.github.io/Hackathon-SketchDate/og.png" />
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content="SketchDate — A little mystery. A real connection." />
	<meta
		name="twitter:description"
		content="Meet through voice and expression. Reveal only when you both say yes."
	/>
	<meta name="twitter:image" content="https://ossca-thorvg.github.io/Hackathon-SketchDate/og.png" />
</svelte:head>

<video
	bind:this={trackerVideo}
	autoplay
	muted
	playsinline
	class="pointer-events-none fixed size-px opacity-0"
	aria-hidden="true"
></video>
<audio bind:this={remoteAudio} autoplay aria-label="Your date's audio"></audio>

<div class="min-h-screen overflow-hidden bg-[#f8f5ef] text-[#241b2f]">
	<div
		class="pointer-events-none fixed inset-0 [background-image:radial-gradient(circle_at_14%_14%,rgba(246,147,178,.24),transparent_28%),radial-gradient(circle_at_88%_22%,rgba(125,100,199,.18),transparent_30%),radial-gradient(circle_at_50%_100%,rgba(255,210,126,.2),transparent_32%)] opacity-60"
	></div>

	<header
		class="relative z-20 mx-auto flex w-full max-w-[1480px] items-center justify-between px-5 py-5 sm:px-8 lg:px-12"
	>
		<button
			type="button"
			class="group flex items-center gap-3"
			onclick={startOver}
			aria-label="Start over"
		>
			<span
				class="grid size-10 rotate-[-4deg] place-items-center rounded-[14px] bg-[#2c2040] text-xl text-white shadow-[0_7px_0_#f39ab5] transition group-hover:rotate-0"
				>✦</span
			>
			<span class="font-serif text-[1.45rem] font-black tracking-[-0.04em]">SketchDate</span>
		</button>
		{#if appState === 'landing'}
			<div class="flex items-center gap-3 sm:gap-7">
				<a
					href="#how-it-works"
					class="hidden text-sm font-bold text-[#6e6372] hover:text-[#30233b] md:block"
					>How it works</a
				>
				<a
					href="#privacy"
					class="hidden text-sm font-bold text-[#6e6372] hover:text-[#30233b] md:block">Privacy</a
				>
				<button
					type="button"
					onclick={beginSetup}
					class="rounded-full bg-[#2d213b] px-4 py-2.5 text-sm font-black text-white shadow-[0_4px_0_#d36b8d] transition hover:-translate-y-0.5 sm:px-5"
					>Start a date</button
				>
			</div>
		{:else}
			<div
				class="hidden items-center gap-2 rounded-full border border-[#dcd4ca] bg-white/70 px-3 py-2 text-[11px] font-bold tracking-[0.16em] text-[#716779] uppercase shadow-sm backdrop-blur sm:flex"
			>
				<span class="size-2 rounded-full bg-[#55a77b]"></span>
				Private by design
			</div>
		{/if}
	</header>

	<main
		class="relative z-10 mx-auto flex min-h-[calc(100vh-88px)] w-full max-w-[1480px] flex-col px-5 pb-8 sm:px-8 lg:px-12"
	>
		{#if appState === 'landing'}
			<div class="flex-1">
				<section
					class="grid min-h-[calc(100vh-112px)] items-center gap-10 py-8 lg:grid-cols-[1.05fr_.95fr] lg:gap-16 lg:py-12"
				>
					<div class="max-w-3xl">
						<p
							class="mb-6 inline-flex items-center gap-2 rounded-full border border-[#e7b7c5] bg-[#fff8fa] px-3 py-1.5 text-xs font-black tracking-[0.14em] text-[#9b4764] uppercase"
						>
							<span class="size-2 rounded-full bg-[#d45f87]"></span> Voice-first blind dates
						</p>
						<h1
							class="font-serif text-[clamp(3.7rem,7.3vw,7.5rem)] leading-[0.88] font-black tracking-[-0.065em] text-[#2d213b]"
						>
							A little mystery.<br /><span class="text-[#d45f87] italic">A real connection.</span>
						</h1>
						<p class="mt-7 max-w-xl text-lg leading-8 text-[#695f70] sm:text-xl">
							Meet one person at a time through voice and expression. Your camera stays private
							until you both choose to be seen.
						</p>
						<div class="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
							<button
								type="button"
								onclick={beginSetup}
								class="rounded-2xl bg-[#2d213b] px-7 py-4 text-base font-black text-white shadow-[0_6px_0_#d36b8d] transition hover:-translate-y-0.5 active:translate-y-1 active:shadow-none"
								>Get ready for a date</button
							>
							<a
								href="#how-it-works"
								class="rounded-2xl px-6 py-4 text-center text-sm font-black text-[#5f5265] hover:bg-white/60"
								>See how it works ↓</a
							>
						</div>
						<div
							class="mt-9 flex flex-wrap items-center gap-x-6 gap-y-3 text-xs font-bold text-[#776b7b]"
						>
							{#each ['No profile needed', 'Camera hidden by default', 'Reveal only together'] as promise (promise)}
								<span class="flex items-center gap-2"
									><span
										class="grid size-5 place-items-center rounded-full bg-[#dff1e6] text-[10px] text-[#347453]"
										>✓</span
									>{promise}</span
								>
							{/each}
						</div>
					</div>

					<div class="relative mx-auto w-full max-w-[590px] pb-8 sm:px-6 lg:pb-0">
						<div
							class="absolute -top-4 -right-2 size-32 rounded-full bg-[#f3c86f]/45 blur-3xl"
						></div>
						<div
							class="absolute -bottom-6 -left-6 size-40 rounded-full bg-[#d88eaa]/35 blur-3xl"
						></div>
						<div
							class="relative rotate-[1.5deg] overflow-hidden rounded-[2.25rem] border border-white/20 bg-[#2d213b] p-5 text-white shadow-[0_35px_90px_rgba(54,35,68,.3)] sm:p-7"
						>
							<div
								class="absolute inset-0 [background-image:radial-gradient(circle_at_20%_10%,rgba(214,119,153,.38),transparent_35%),radial-gradient(circle_at_90%_90%,rgba(112,91,151,.5),transparent_42%)] opacity-80"
							></div>
							<div class="relative flex items-center justify-between">
								<div>
									<p class="text-[10px] font-black tracking-[0.18em] text-white/50 uppercase">
										SketchDate session
									</p>
									<p class="mt-1 font-serif text-2xl font-black">Voice first</p>
								</div>
								<span
									class="flex items-center gap-2 rounded-full bg-white/10 px-3 py-2 text-xs font-bold backdrop-blur"
									><span class="size-2 animate-pulse rounded-full bg-[#73dda2]"></span>Live</span
								>
							</div>
							<div class="relative my-10 flex items-center justify-center gap-2 sm:my-14">
								{#each [24, 42, 68, 38, 82, 54, 94, 62, 36, 74, 48, 26, 58, 34, 18] as height, index (index)}
									<span
										class="w-1.5 rounded-full bg-gradient-to-t from-[#e36e99] to-[#ffd2df]"
										style={`height: ${height}px; opacity: ${0.55 + (index % 4) * 0.12}`}
									></span>
								{/each}
							</div>
							<div class="relative grid grid-cols-2 gap-3">
								<div class="rounded-2xl border border-white/10 bg-white/8 p-4 backdrop-blur">
									<div class="mb-8 flex items-center justify-between">
										<span
											class="grid size-10 place-items-center rounded-full bg-[#e4a1b8] font-serif text-lg font-black text-[#3a2338]"
											>Y</span
										><span class="text-xs text-white/45">You</span>
									</div>
									<p class="text-sm font-bold">Camera hidden</p>
									<p class="mt-1 text-xs text-white/45">Expressions stay on-device</p>
								</div>
								<div class="rounded-2xl border border-white/10 bg-white/8 p-4 backdrop-blur">
									<div class="mb-8 flex items-center justify-between">
										<span
											class="grid size-10 place-items-center rounded-full bg-[#9b8cc4] font-serif text-lg font-black text-[#281f36]"
											>?</span
										><span class="text-xs text-white/45">Your date</span>
									</div>
									<p class="text-sm font-bold">One real person</p>
									<p class="mt-1 text-xs text-white/45">No swiping, just talking</p>
								</div>
							</div>
						</div>
						<div
							class="relative -mt-5 ml-auto w-[78%] -rotate-2 rounded-2xl border border-[#dfd5dc] bg-white p-4 shadow-[0_16px_45px_rgba(54,35,68,.16)] sm:-mr-3 sm:w-[66%]"
						>
							<div class="flex items-center gap-3">
								<span
									class="grid size-10 shrink-0 place-items-center rounded-xl bg-[#f7e4ea] text-lg"
									>♡</span
								>
								<div>
									<p class="text-xs font-black tracking-[0.12em] text-[#a95573] uppercase">
										Mutual reveal
									</p>
									<p class="mt-0.5 text-sm font-bold">Video opens only after two yeses.</p>
								</div>
							</div>
						</div>
					</div>
				</section>

				<section id="how-it-works" class="scroll-mt-8 border-t border-[#ded6ce] py-20 sm:py-28">
					<div class="mb-10 max-w-2xl">
						<p class="text-xs font-black tracking-[0.16em] text-[#a95573] uppercase">
							A softer way to meet
						</p>
						<h2 class="mt-3 font-serif text-4xl font-black tracking-[-0.045em] sm:text-5xl">
							Two moments. No performance.
						</h2>
					</div>
					<div class="grid gap-4 md:grid-cols-2">
						{#each [{ number: '01', title: 'Set up and meet', text: 'Customize your character, check your camera and microphone, then meet one person by voice.', tone: 'bg-[#f4dce4]' }, { number: '02', title: 'Choose together', text: 'After the date, answer privately. Video opens only when both people say yes.', tone: 'bg-[#e5def1]' }] as step (step.number)}
							<article
								class="rounded-[1.75rem] border border-white bg-white/65 p-6 shadow-sm backdrop-blur sm:p-8"
							>
								<span
									class={[
										'grid size-11 place-items-center rounded-2xl text-xs font-black',
										step.tone
									]}>{step.number}</span
								>
								<h3 class="mt-8 font-serif text-2xl font-black">{step.title}</h3>
								<p class="mt-3 leading-7 text-[#746978]">{step.text}</p>
							</article>
						{/each}
					</div>
				</section>

				<section
					id="privacy"
					class="mb-8 overflow-hidden rounded-[2.25rem] bg-[#2d213b] px-6 py-12 text-white sm:px-12 sm:py-16"
				>
					<div class="grid items-center gap-8 lg:grid-cols-[1fr_auto] lg:gap-16">
						<div class="max-w-3xl">
							<p class="text-xs font-black tracking-[0.16em] text-[#f0a9bf] uppercase">
								Privacy is the premise
							</p>
							<h2 class="mt-3 font-serif text-4xl font-black tracking-[-0.045em] sm:text-5xl">
								Your face is not the price of admission.
							</h2>
							<p class="mt-5 max-w-2xl leading-7 text-white/65">
								Camera frames are processed locally for expression tracking and are not sent during
								the character date. A video track is added only after mutual consent.
							</p>
						</div>
						<button
							type="button"
							onclick={beginSetup}
							class="rounded-2xl bg-[#f5a0b9] px-7 py-4 font-black text-[#321e35] shadow-[0_6px_0_#a54c6b] transition hover:-translate-y-0.5"
							>Start with a sketch</button
						>
					</div>
				</section>
			</div>
		{:else if appState === 'media-ready'}
			<section
				class="mx-auto grid w-full max-w-7xl flex-1 items-start gap-6 py-4 lg:grid-cols-[1.04fr_.96fr] lg:py-6"
			>
				<div
					bind:clientWidth={setupStageWidth}
					bind:clientHeight={setupStageHeight}
					class="relative mx-auto aspect-[5/4] w-full max-w-[570px] overflow-hidden rounded-[2rem] bg-[#2d213b] shadow-[0_24px_70px_rgba(64,42,78,.2)] lg:sticky lg:top-4 lg:aspect-auto lg:h-[calc(100vh-128px)] lg:max-h-[640px] lg:min-h-[480px]"
				>
					<div
						class="absolute inset-0 [background-image:radial-gradient(circle_at_50%_35%,#76618f_0,transparent_44%),linear-gradient(145deg,#21182e,#38284a)]"
					></div>
					<CharacterRenderer
						pose={localPose}
						parts={selectedParts}
						width={setupStageWidth || 700}
						height={setupStageHeight || 560}
						label="Your character following your face"
						class="absolute inset-0 h-full w-full"
					/>
					<div
						class="absolute top-5 left-5 flex items-center gap-2 rounded-full bg-black/25 px-3 py-2 text-xs font-bold text-white backdrop-blur"
					>
						<span
							class={[
								'size-2 rounded-full',
								faceDetected ? 'bg-[#68d89b]' : 'animate-pulse bg-[#f5c46b]'
							]}
						></span>
						{faceDetected ? 'Face found' : 'Looking for your face'}
					</div>
					<button
						type="button"
						onclick={startOver}
						class="absolute top-5 right-5 rounded-full bg-black/25 px-3 py-2 text-xs font-bold text-white/80 backdrop-blur transition hover:bg-black/40 hover:text-white"
						>Back</button
					>
				</div>
				<div class="mx-auto w-full max-w-[34rem]">
					<p class="text-xs font-black tracking-[0.16em] text-[#a95573] uppercase">
						02 · Get comfortable
					</p>
					<h1
						class="mt-2 font-serif text-4xl leading-none font-black tracking-[-0.05em] sm:text-5xl"
					>
						Your look. Your setup.
					</h1>
					<p class="mt-3 text-sm leading-6 text-[#706675] sm:text-base">
						Choose your character while we privately check your expression, camera, and sound.
					</p>
					<div class="mt-4">
						<CharacterPicker bind:selectedParts />
					</div>
					<div class="mt-3 grid grid-cols-3 gap-2">
						{#each [{ label: 'Face model', detail: modelReady ? 'Ready' : 'Loading safely…', ready: modelReady }, { label: 'Camera', detail: cameraReady ? 'On · stays local' : 'Waiting for access', ready: cameraReady }, { label: 'Microphone', detail: microphoneReady ? 'Ready for voice' : 'Waiting for access', ready: microphoneReady }] as item (item.label)}
							<div
								class="flex min-w-0 items-center justify-between rounded-xl border border-[#e4ddd4] bg-white/75 p-2.5 shadow-sm"
							>
								<div>
									<p class="truncate text-xs font-bold sm:text-sm">{item.label}</p>
									<p class="mt-0.5 truncate text-[11px] text-[#817684]">{item.detail}</p>
								</div>
								<span
									class={[
										'ml-1 grid size-6 shrink-0 place-items-center rounded-full text-xs font-black',
										item.ready ? 'bg-[#dff3e7] text-[#27734d]' : 'bg-[#eee9e1] text-[#918796]'
									]}>{item.ready ? '✓' : '·'}</span
								>
							</div>
						{/each}
					</div>
					<div class="sticky bottom-0 z-20 mt-3 bg-[#f8f5ef]/92 pt-2 pb-1 backdrop-blur-md">
						<button
							type="button"
							disabled={!readyToMatch || preparing}
							onclick={startMatching}
							class="w-full rounded-2xl bg-[#2d213b] px-6 py-3.5 font-black text-white shadow-[0_5px_0_#d36b8d] transition enabled:hover:-translate-y-0.5 disabled:cursor-not-allowed disabled:opacity-40"
							>{preparing
								? 'Getting camera and sound ready…'
								: faceDetected
									? 'Start matching'
									: 'Move into the camera frame'}</button
						>
					</div>
				</div>
			</section>
		{:else if appState === 'matching'}
			<section class="grid flex-1 place-items-center py-10 text-center">
				<div class="w-full max-w-lg">
					<div class="relative mx-auto mb-9 aspect-square w-64">
						<div
							class="absolute inset-0 animate-[ping_2.2s_ease-out_infinite] rounded-full border border-[#df9bb1]"
						></div>
						<div
							class="absolute inset-5 animate-[ping_2.2s_.6s_ease-out_infinite] rounded-full border border-[#9a83ba]"
						></div>
						<div class="absolute inset-10 overflow-hidden rounded-full bg-[#2d213b] shadow-2xl">
							<CharacterRenderer
								pose={localPose}
								parts={selectedParts}
								width={400}
								height={400}
								label="Your waiting character"
								class="absolute inset-0 h-full w-full"
							/>
						</div>
					</div>
					<p class="text-xs font-black tracking-[0.17em] text-[#a95573] uppercase">
						Finding your date
					</p>
					<h1 class="mt-3 font-serif text-5xl font-black tracking-[-0.05em]">
						Someone new is<br />just a moment away.
					</h1>
					<p class="mt-5 text-[#756a79]">{connectionStatus}</p>
					<button
						type="button"
						onclick={cancelMatching}
						class="mt-8 rounded-full border border-[#cabfca] bg-white/70 px-5 py-2.5 text-sm font-bold"
						>Cancel</button
					>
				</div>
			</section>
		{:else if appState === 'avatar-date'}
			<section class="mx-auto flex w-full max-w-5xl flex-1 flex-col py-4">
				<div class="mb-4 flex items-center justify-between">
					<div>
						<p class="text-xs font-black tracking-[0.15em] text-[#a95573] uppercase">
							Character date
						</p>
						<p class="mt-1 text-sm text-[#776c7a]">{connectionStatus}</p>
					</div>
					<div
						class="rounded-full border border-[#dfd6dc] bg-white/80 px-4 py-2 font-mono text-sm font-bold tabular-nums"
					>
						{formatTime(countdown)}
					</div>
				</div>
				<div
					class="relative min-h-0 flex-1 overflow-hidden rounded-[2.25rem] bg-[#2d213b] shadow-[0_30px_80px_rgba(64,42,78,.24)]"
				>
					<div
						class="absolute inset-0 [background-image:radial-gradient(circle_at_50%_36%,#725c8d_0,transparent_43%),linear-gradient(150deg,#20162d,#3a294d)]"
					></div>
					<CharacterRenderer
						pose={remotePose}
						parts={remoteParts}
						width={900}
						height={700}
						label="Your date's animated character"
						class="absolute inset-0 h-full w-full"
					/>
					<div
						class="absolute top-5 left-5 flex items-center gap-2 rounded-full bg-black/25 px-3 py-2 text-xs font-bold text-white backdrop-blur"
					>
						<span
							class={[
								'size-2 rounded-full',
								connectionStatus === 'Live and private'
									? 'bg-[#68d89b]'
									: 'animate-pulse bg-[#f5c46b]'
							]}
						></span>{connectionStatus}
					</div>
					<div
						class="absolute right-4 bottom-4 aspect-[4/5] w-24 overflow-hidden rounded-2xl border-2 border-white/20 bg-[#59466f] shadow-xl sm:right-6 sm:bottom-6 sm:w-32"
					>
						<CharacterRenderer
							pose={localPose}
							parts={selectedParts}
							width={240}
							height={300}
							label="Your character"
							class="absolute inset-0 h-full w-full"
						/>
					</div>
				</div>
				<div class="mt-5 flex items-center justify-center gap-3">
					<button
						type="button"
						onclick={toggleMute}
						class={[
							'rounded-full px-5 py-3 text-sm font-black transition',
							muted ? 'bg-[#f1dfe5] text-[#963d5e]' : 'bg-white text-[#392c42] shadow-sm'
						]}>{muted ? 'Unmute' : 'Mute'}</button
					>
					<button
						type="button"
						onclick={leaveDate}
						class="rounded-full bg-[#d95f67] px-5 py-3 text-sm font-black text-white shadow-sm"
						>End date</button
					>
				</div>
			</section>
		{:else if appState === 'decision'}
			<section class="grid flex-1 place-items-center py-8">
				<div
					class="w-full max-w-xl rounded-[2.25rem] border border-white bg-white/80 p-7 text-center shadow-[0_30px_90px_rgba(64,42,78,.16)] backdrop-blur sm:p-12"
				>
					<div class="mx-auto grid size-14 place-items-center rounded-2xl bg-[#f7e4ea] text-2xl">
						♡
					</div>
					<p class="mt-6 text-xs font-black tracking-[0.16em] text-[#a95573] uppercase">
						Your choice is private
					</p>
					<h1 class="mt-3 font-serif text-5xl leading-[1.02] font-black tracking-[-0.05em]">
						Ready to meet<br />face to face?
					</h1>
					<p class="mx-auto mt-5 max-w-md leading-7 text-[#756a79]">
						Your answer is never shown to the other person. Video opens only if you both choose yes.
					</p>
					{#if choice}
						<div class="mt-8 rounded-2xl bg-[#f3eee8] p-5">
							<p class="font-black">Choice locked in</p>
							<p class="mt-1 text-sm text-[#756a79]">
								Waiting privately for the result · {countdown}s
							</p>
						</div>
					{:else}
						<div class="mt-8 grid gap-3 sm:grid-cols-2">
							<button
								type="button"
								onclick={() => submitDecision('reveal')}
								class="rounded-2xl bg-[#2d213b] px-5 py-4 font-black text-white shadow-[0_5px_0_#d36b8d] transition hover:-translate-y-0.5"
								>Yes, reveal video</button
							>
							<button
								type="button"
								onclick={() => submitDecision('end')}
								class="rounded-2xl border border-[#d9cfd7] bg-white px-5 py-4 font-black text-[#554a59]"
								>End the date</button
							>
						</div>
						<p class="mt-5 font-mono text-sm font-bold text-[#8a7d8d]">
							{countdown} seconds to choose
						</p>
					{/if}
				</div>
			</section>
		{:else if appState === 'video-date'}
			<section class="mx-auto flex w-full max-w-5xl flex-1 flex-col py-4">
				<div class="mb-4 flex items-center justify-between">
					<div>
						<p class="text-xs font-black tracking-[0.15em] text-[#a95573] uppercase">
							Face to face
						</p>
						<p class="mt-1 text-sm text-[#776c7a]">You both said yes</p>
					</div>
					<span class="rounded-full bg-[#deefe5] px-3 py-2 text-xs font-black text-[#2e7651]"
						>Video revealed</span
					>
				</div>
				<div
					class="relative min-h-0 flex-1 overflow-hidden rounded-[2.25rem] bg-[#21192a] shadow-[0_30px_80px_rgba(64,42,78,.24)]"
				>
					<video
						bind:this={remoteVideo}
						autoplay
						muted
						playsinline
						onloadeddata={() => (videoReady = true)}
						class={[
							'absolute inset-0 h-full w-full object-cover transition duration-700',
							videoReady ? 'opacity-100' : 'opacity-0'
						]}
						aria-label="Your date's video"
					></video>
					{#if !videoReady}<div
							class="absolute inset-0 grid place-items-center text-center text-white"
						>
							<div>
								<div
									class="mx-auto mb-4 size-9 animate-spin rounded-full border-2 border-white/25 border-t-white"
								></div>
								<p class="font-bold">Opening video securely…</p>
							</div>
						</div>{/if}
					<video
						bind:this={selfVideo}
						autoplay
						muted
						playsinline
						class="absolute right-4 bottom-4 aspect-[4/5] w-24 -scale-x-100 rounded-2xl border-2 border-white/30 bg-black object-cover shadow-xl sm:right-6 sm:bottom-6 sm:w-32"
						aria-label="Your video preview"
					></video>
				</div>
				<div class="mt-5 flex items-center justify-center gap-3">
					<button
						type="button"
						onclick={toggleMute}
						class={[
							'rounded-full px-5 py-3 text-sm font-black',
							muted ? 'bg-[#f1dfe5] text-[#963d5e]' : 'bg-white shadow-sm'
						]}>{muted ? 'Unmute' : 'Mute'}</button
					><button
						type="button"
						onclick={leaveDate}
						class="rounded-full bg-[#d95f67] px-5 py-3 text-sm font-black text-white"
						>End date</button
					>
				</div>
			</section>
		{:else}
			<section class="grid flex-1 place-items-center py-10 text-center">
				<div class="w-full max-w-lg">
					<div
						class={[
							'mx-auto grid size-16 place-items-center rounded-2xl text-2xl',
							isError ? 'bg-[#f6dce2] text-[#a93f5d]' : 'bg-[#e5dff0] text-[#5e4780]'
						]}
					>
						{isError ? '!' : '✦'}
					</div>
					<p class="mt-6 text-xs font-black tracking-[0.16em] text-[#a95573] uppercase">
						{isError ? 'Something went wrong' : 'Date complete'}
					</p>
					<h1 class="mt-3 font-serif text-5xl font-black tracking-[-0.05em]">
						{isError ? 'Let’s try that again.' : 'Thanks for showing up.'}
					</h1>
					<p class="mx-auto mt-5 max-w-md leading-7 text-[#756a79]">{endedReason}</p>
					<button
						type="button"
						onclick={startOver}
						class="mt-8 rounded-2xl bg-[#2d213b] px-7 py-4 font-black text-white shadow-[0_5px_0_#d36b8d]"
						>Start over</button
					>
				</div>
			</section>
		{/if}
	</main>
</div>
