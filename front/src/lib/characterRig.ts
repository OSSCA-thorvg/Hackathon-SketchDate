/**
 * Transport-neutral character pose.
 *
 * Positions, gaze, yaw, pitch and expression values are normalized so this shape can be
 * produced by MediaPipe, a recording, or a remote WebRTC peer without camera dimensions.
 */
export type RigPose = {
	/** Horizontal face position, from -1 (left) to 1 (right). */
	x: number;
	/** Vertical face position, from -1 (top) to 1 (bottom). */
	y: number;
	/** Relative face size where 1 is the authored character size. */
	scaleX: number;
	scaleY: number;
	/** Roll in degrees. */
	rotation: number;
	/** Head direction, normalized to -1…1. */
	yaw: number;
	pitch: number;
	/** Eye openness, normalized to 0…1. */
	leftEyeOpen: number;
	rightEyeOpen: number;
	/** Gaze direction, normalized to -1…1. */
	leftGazeX: number;
	leftGazeY: number;
	rightGazeX: number;
	rightGazeY: number;
	/** Brow lift, normalized to -1…1. */
	leftBrowLift: number;
	rightBrowLift: number;
	/** Mouth openness is 0…1 and smile is -1 (frown) to 1 (smile). */
	mouthOpen: number;
	smile: number;
};

export const NEUTRAL_RIG_POSE: RigPose = {
	x: 0,
	y: 0,
	scaleX: 1,
	scaleY: 1,
	rotation: 0,
	yaw: 0,
	pitch: 0,
	leftEyeOpen: 1,
	rightEyeOpen: 1,
	leftGazeX: 0,
	leftGazeY: 0,
	rightGazeX: 0,
	rightGazeY: 0,
	leftBrowLift: 0,
	rightBrowLift: 0,
	mouthOpen: 0.5,
	smile: 0
};
