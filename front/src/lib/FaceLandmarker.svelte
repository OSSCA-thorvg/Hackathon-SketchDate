<script lang="ts">
	import CharacterPicker from '$lib/CharacterPicker.svelte';
	import FaceCharacterStage from '$lib/FaceCharacterStage.svelte';
	import { DEFAULT_PARTS, type PartSelection } from '$lib/avatarParts';
	import type { RigPose } from '$lib/characterRig';
	import { onMount } from 'svelte';

	type FaceLandmarkerInstance = import('@mediapipe/tasks-vision').FaceLandmarker;
	type Blendshape = Pick<import('@mediapipe/tasks-vision').Category, 'categoryName' | 'score'>;
	type Landmark = { x: number; y: number; z?: number };
	type Point = { x: number; y: number };

	const MEDIAPIPE_VERSION = '1.0.1';

	const WASM_URL = `https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@${MEDIAPIPE_VERSION}/wasm`;

	const MODEL_URL =
		'https://storage.googleapis.com/mediapipe-models/' +
		'face_landmarker/face_landmarker/float16/1/' +
		'face_landmarker.task';

	let videoElement = $state<HTMLVideoElement>(undefined!);

	let faceLandmarker: FaceLandmarkerInstance | null = null;

	let mediaStream: MediaStream | null = null;
	let animationFrameId: number | null = null;

	let modelReady = $state(false);
	let running = $state(false);

	let status = $state('Loading MediaPipe model…');
	let selectedParts = $state<PartSelection>({ ...DEFAULT_PARTS });
	let avatarFrame = $state<RigPose | null>(null);
	let smoothedPose: RigPose | null = null;

	let lastVideoTime = -1;
	let previousFaceCount = $state(-1);

	onMount(() => {
		let destroyed = false;

		void initializeMediaPipe();

		async function initializeMediaPipe(): Promise<void> {
			try {
				const { FaceLandmarker, FilesetResolver } = await import('@mediapipe/tasks-vision');

				const vision = await FilesetResolver.forVisionTasks(WASM_URL);

				const landmarker = await FaceLandmarker.createFromOptions(vision, {
					baseOptions: {
						modelAssetPath: MODEL_URL
					},
					runningMode: 'VIDEO',
					numFaces: 1,
					minFaceDetectionConfidence: 0.5,
					minFacePresenceConfidence: 0.5,
					minTrackingConfidence: 0.5,
					outputFaceBlendshapes: true,
					outputFacialTransformationMatrixes: false
				});

				if (destroyed) {
					landmarker.close();
					return;
				}

				faceLandmarker = landmarker;

				modelReady = true;
				status = 'Starting camera…';

				try {
					await startCamera();
				} catch (error) {
					console.error(error);
					stopCamera();
					status = getErrorMessage(error);
				}
			} catch (error) {
				console.error(error);

				if (!destroyed) {
					status = getErrorMessage(error);
				}
			}
		}

		return () => {
			destroyed = true;

			stopCamera();

			faceLandmarker?.close();
			faceLandmarker = null;
		};
	});

	async function startCamera(): Promise<void> {
		if (!faceLandmarker) {
			throw new Error('MediaPipe is not ready yet.');
		}

		if (!navigator.mediaDevices?.getUserMedia) {
			throw new Error('Webcam access is not supported by this browser.');
		}

		mediaStream = await navigator.mediaDevices.getUserMedia({
			audio: false,
			video: {
				facingMode: 'user',
				width: {
					ideal: 1280
				},
				height: {
					ideal: 720
				}
			}
		});

		videoElement.srcObject = mediaStream;

		await waitForVideoMetadata(videoElement);
		await videoElement.play();

		running = true;
		lastVideoTime = -1;
		previousFaceCount = -1;
		status = 'Looking for a face…';

		animationFrameId = requestAnimationFrame(renderLoop);
	}

	function stopCamera(): void {
		running = false;

		if (animationFrameId !== null) {
			cancelAnimationFrame(animationFrameId);
			animationFrameId = null;
		}

		mediaStream?.getTracks().forEach((track) => {
			track.stop();
		});

		mediaStream = null;

		if (videoElement) {
			videoElement.srcObject = null;
		}

		avatarFrame = null;
		smoothedPose = null;

		if (modelReady) {
			status = 'Model ready';
		}
	}

	function renderLoop(): void {
		if (!running || !faceLandmarker) {
			return;
		}

		if (
			videoElement.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA &&
			videoElement.currentTime !== lastVideoTime
		) {
			lastVideoTime = videoElement.currentTime;

			const result = faceLandmarker.detectForVideo(videoElement, performance.now());

			if (result.faceLandmarks[0]) {
				const targetPose = createRigPose(
					result.faceLandmarks[0],
					videoElement.videoWidth,
					videoElement.videoHeight,
					result.faceBlendshapes[0]?.categories ?? []
				);

				smoothedPose = smoothedPose ? smoothPose(smoothedPose, targetPose) : targetPose;
				avatarFrame = smoothedPose;
			} else {
				avatarFrame = null;
				smoothedPose = null;
			}

			if (result.faceLandmarks.length !== previousFaceCount) {
				previousFaceCount = result.faceLandmarks.length;

				status = previousFaceCount > 0 ? `${previousFaceCount} face detected` : 'No face detected';
			}
		}

		animationFrameId = requestAnimationFrame(renderLoop);
	}

	function createRigPose(
		landmarks: Landmark[],
		width: number,
		height: number,
		blendshapes: Blendshape[]
	): RigPose {
		const point = (index: number): Point => ({
			x: landmarks[index].x * width,
			y: landmarks[index].y * height
		});
		const faceLeft = point(234);
		const faceRight = point(454);
		const forehead = point(10);
		const chin = point(152);
		const nose = point(1);
		const leftEyeOuter = point(33);
		const leftEyeInner = point(133);
		const rightEyeInner = point(362);
		const rightEyeOuter = point(263);
		const leftEyeCenter = midpoint(leftEyeOuter, leftEyeInner);
		const rightEyeCenter = midpoint(rightEyeInner, rightEyeOuter);
		const faceCenter = midpoint(forehead, chin);
		const faceWidth = distance(faceLeft, faceRight);
		const faceHeight = distance(forehead, chin);
		const leftEyeWidth = distance(leftEyeOuter, leftEyeInner);
		const rightEyeWidth = distance(rightEyeInner, rightEyeOuter);
		const leftIris = point(468);
		const rightIris = point(473);
		const mouthLeft = point(61);
		const mouthRight = point(291);
		const mouthCenter = midpoint(point(13), point(14));
		const mouthWidth = distance(mouthLeft, mouthRight);
		const mouthCornerY = (mouthLeft.y + mouthRight.y) / 2;

		const eyeOpen = (upper: Point, lower: Point, eyeWidth: number) =>
			clamp((distance(upper, lower) / eyeWidth - 0.055) / 0.19, 0.04, 1);
		const gaze = (iris: Point, center: Point, eyeWidth: number) => ({
			x: clamp(((iris.x - center.x) / eyeWidth) * 2.8, -1, 1),
			y: clamp(((iris.y - center.y) / eyeWidth) * 3.4, -1, 1)
		});
		const leftGaze = gaze(leftIris, leftEyeCenter, leftEyeWidth);
		const rightGaze = gaze(rightIris, rightEyeCenter, rightEyeWidth);
		const browLift = (brow: Point, eye: Point) =>
			clamp(((eye.y - brow.y) / faceHeight - 0.055) / 0.07, -0.35, 1);
		const blendshapeScore = (categoryName: string) =>
			blendshapes.find((blendshape) => blendshape.categoryName === categoryName)?.score ?? 0;
		const geometricSmile = clamp(
			((mouthCenter.y - mouthCornerY) / mouthWidth - 0.01) / 0.11,
			-1,
			1
		);
		const smileBlendshape = clamp(
			((blendshapeScore('mouthSmileLeft') + blendshapeScore('mouthSmileRight')) / 2 - 0.06) / 0.5,
			0,
			1
		);
		const frownBlendshape = clamp(
			((blendshapeScore('mouthFrownLeft') + blendshapeScore('mouthFrownRight')) / 2 - 0.08) / 0.45,
			0,
			1
		);
		const smile =
			blendshapes.length > 0
				? clamp(smileBlendshape * 0.85 - frownBlendshape * 0.85 + geometricSmile * 0.15, -1, 1)
				: geometricSmile;

		return {
			x: clamp((faceCenter.x / width - 0.5) * 2, -1, 1),
			y: clamp((faceCenter.y / height - 0.5) * 2, -1, 1),
			scaleX: clamp(faceWidth / (width * 0.15), 0.35, 4),
			scaleY: clamp(faceHeight / (height * 0.32), 0.35, 4),
			rotation:
				(Math.atan2(rightEyeCenter.y - leftEyeCenter.y, rightEyeCenter.x - leftEyeCenter.x) * 180) /
				Math.PI,
			yaw: clamp(((nose.x - faceCenter.x) / faceWidth) * 4, -1, 1),
			pitch: clamp(((nose.y - faceCenter.y) / faceHeight - 0.08) * 4, -1, 1),
			leftEyeOpen: eyeOpen(point(159), point(145), leftEyeWidth),
			rightEyeOpen: eyeOpen(point(386), point(374), rightEyeWidth),
			leftGazeX: leftGaze.x,
			leftGazeY: leftGaze.y,
			rightGazeX: rightGaze.x,
			rightGazeY: rightGaze.y,
			leftBrowLift: browLift(point(105), leftEyeCenter),
			rightBrowLift: browLift(point(334), rightEyeCenter),
			mouthOpen: clamp((distance(point(13), point(14)) / mouthWidth - 0.025) / 0.42, 0, 1),
			smile
		};
	}

	function smoothPose(previous: RigPose, target: RigPose): RigPose {
		const motionAlpha = 0.38;
		const expressionAlpha = 0.24;
		const lerp = (from: number, to: number, alpha: number) => from + (to - from) * alpha;

		return {
			x: lerp(previous.x, target.x, motionAlpha),
			y: lerp(previous.y, target.y, motionAlpha),
			scaleX: lerp(previous.scaleX, target.scaleX, motionAlpha),
			scaleY: lerp(previous.scaleY, target.scaleY, motionAlpha),
			rotation: lerp(previous.rotation, target.rotation, motionAlpha),
			yaw: lerp(previous.yaw, target.yaw, expressionAlpha),
			pitch: lerp(previous.pitch, target.pitch, expressionAlpha),
			leftEyeOpen: lerp(previous.leftEyeOpen, target.leftEyeOpen, expressionAlpha),
			rightEyeOpen: lerp(previous.rightEyeOpen, target.rightEyeOpen, expressionAlpha),
			leftGazeX: lerp(previous.leftGazeX, target.leftGazeX, expressionAlpha),
			leftGazeY: lerp(previous.leftGazeY, target.leftGazeY, expressionAlpha),
			rightGazeX: lerp(previous.rightGazeX, target.rightGazeX, expressionAlpha),
			rightGazeY: lerp(previous.rightGazeY, target.rightGazeY, expressionAlpha),
			leftBrowLift: lerp(previous.leftBrowLift, target.leftBrowLift, expressionAlpha),
			rightBrowLift: lerp(previous.rightBrowLift, target.rightBrowLift, expressionAlpha),
			mouthOpen: lerp(previous.mouthOpen, target.mouthOpen, expressionAlpha),
			smile: lerp(previous.smile, target.smile, expressionAlpha)
		};
	}

	function midpoint(first: Point, second: Point): Point {
		return { x: (first.x + second.x) / 2, y: (first.y + second.y) / 2 };
	}

	function distance(first: Point, second: Point): number {
		return Math.hypot(second.x - first.x, second.y - first.y);
	}

	function clamp(value: number, minimum: number, maximum: number): number {
		return Math.min(maximum, Math.max(minimum, value));
	}

	function waitForVideoMetadata(video: HTMLVideoElement): Promise<void> {
		if (video.readyState >= HTMLMediaElement.HAVE_METADATA) {
			return Promise.resolve();
		}

		return new Promise((resolve, reject) => {
			function handleLoadedMetadata(): void {
				cleanup();
				resolve();
			}

			function handleError(): void {
				cleanup();
				reject(new Error('Could not load the webcam stream.'));
			}

			function cleanup(): void {
				video.removeEventListener('loadedmetadata', handleLoadedMetadata);

				video.removeEventListener('error', handleError);
			}

			video.addEventListener('loadedmetadata', handleLoadedMetadata);

			video.addEventListener('error', handleError);
		});
	}

	function getErrorMessage(error: unknown): string {
		if (error instanceof DOMException) {
			if (error.name === 'NotAllowedError') {
				return 'Webcam permission was denied.';
			}

			if (error.name === 'NotFoundError') {
				return 'No webcam was found.';
			}
		}

		return error instanceof Error ? error.message : String(error);
	}
</script>

<svelte:head>
	<title>MediaPipe Face Landmarks</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-gray-50">
	<main class="mx-auto w-full max-w-6xl px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
		<div class="grid items-start gap-6 lg:grid-cols-[minmax(0,2fr)_minmax(17rem,1fr)] lg:gap-8">
			<FaceCharacterStage
				pose={avatarFrame}
				parts={selectedParts}
				{running}
				{modelReady}
				cameraStatus={status}
				faceCount={previousFaceCount}
				bind:videoElement
			/>

			<CharacterPicker bind:selectedParts />
		</div>
	</main>
</div>
