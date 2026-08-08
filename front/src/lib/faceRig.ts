import type { RigPose } from '$lib/characterRig';

type Blendshape = { categoryName: string; score: number };
type Landmark = { x: number; y: number; z?: number };
type Point = { x: number; y: number };

export function createRigPose(
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
	const geometricSmile = clamp(((mouthCenter.y - mouthCornerY) / mouthWidth - 0.01) / 0.11, -1, 1);
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

export function smoothRigPose(previous: RigPose, target: RigPose): RigPose {
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
