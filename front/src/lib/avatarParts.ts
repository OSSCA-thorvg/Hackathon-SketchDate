export type HeadId =
	| 'peach'
	| 'cream'
	| 'outline'
	| 'lavender'
	| 'mint'
	| 'rose'
	| 'sky'
	| 'cocoa'
	| 'robot'
	| 'sunny';
export type EyesId =
	| 'sparkle'
	| 'soft'
	| 'dots'
	| 'stars'
	| 'sleepy'
	| 'wide'
	| 'wink'
	| 'hearts'
	| 'pixel'
	| 'lashes';
export type NoseId =
	| 'curve'
	| 'cat'
	| 'none'
	| 'button'
	| 'triangle'
	| 'dot'
	| 'oval'
	| 'diamond'
	| 'pig'
	| 'freckles';
export type MouthId =
	| 'berry'
	| 'rose'
	| 'line'
	| 'toothy'
	| 'heart'
	| 'coral'
	| 'bubble'
	| 'vampire'
	| 'gloss'
	| 'o-mouth';
export type AccessoryId =
	| 'helmet'
	| 'cat-hood'
	| 'none'
	| 'flowers'
	| 'glasses'
	| 'beanie'
	| 'crown'
	| 'bow'
	| 'earrings'
	| 'monocle';
export type PartKey = 'head' | 'eyes' | 'nose' | 'mouth' | 'accessory';

export type PartSelection = {
	head: HeadId;
	eyes: EyesId;
	nose: NoseId;
	mouth: MouthId;
	accessory: AccessoryId;
};

export type PartOption = { id: string; name: string };
export type PartGroup = { id: PartKey; name: string; options: readonly PartOption[] };

export const PART_GROUPS: readonly PartGroup[] = [
	{
		id: 'head',
		name: 'Head',
		options: [
			{ id: 'peach', name: 'Peach' },
			{ id: 'cream', name: 'Cream' },
			{ id: 'outline', name: 'Outline' },
			{ id: 'lavender', name: 'Lavender' },
			{ id: 'mint', name: 'Mint' },
			{ id: 'rose', name: 'Rose' },
			{ id: 'sky', name: 'Sky' },
			{ id: 'cocoa', name: 'Cocoa' },
			{ id: 'robot', name: 'Robot' },
			{ id: 'sunny', name: 'Sunny' }
		]
	},
	{
		id: 'eyes',
		name: 'Eyes',
		options: [
			{ id: 'sparkle', name: 'Sparkle' },
			{ id: 'soft', name: 'Soft' },
			{ id: 'dots', name: 'Dots' },
			{ id: 'stars', name: 'Stars' },
			{ id: 'sleepy', name: 'Sleepy' },
			{ id: 'wide', name: 'Wide' },
			{ id: 'wink', name: 'Wink' },
			{ id: 'hearts', name: 'Hearts' },
			{ id: 'pixel', name: 'Pixel' },
			{ id: 'lashes', name: 'Lashes' }
		]
	},
	{
		id: 'nose',
		name: 'Nose',
		options: [
			{ id: 'curve', name: 'Curve' },
			{ id: 'cat', name: 'Cat' },
			{ id: 'none', name: 'None' },
			{ id: 'button', name: 'Button' },
			{ id: 'triangle', name: 'Triangle' },
			{ id: 'dot', name: 'Dot' },
			{ id: 'oval', name: 'Oval' },
			{ id: 'diamond', name: 'Diamond' },
			{ id: 'pig', name: 'Pig' },
			{ id: 'freckles', name: 'Freckles' }
		]
	},
	{
		id: 'mouth',
		name: 'Mouth',
		options: [
			{ id: 'berry', name: 'Berry' },
			{ id: 'rose', name: 'Rose' },
			{ id: 'line', name: 'Line' },
			{ id: 'toothy', name: 'Toothy' },
			{ id: 'heart', name: 'Heart' },
			{ id: 'coral', name: 'Coral' },
			{ id: 'bubble', name: 'Bubble' },
			{ id: 'vampire', name: 'Vampire' },
			{ id: 'gloss', name: 'Gloss' },
			{ id: 'o-mouth', name: 'O mouth' }
		]
	},
	{
		id: 'accessory',
		name: 'Accessory',
		options: [
			{ id: 'helmet', name: 'Helmet' },
			{ id: 'cat-hood', name: 'Cat hood' },
			{ id: 'none', name: 'None' },
			{ id: 'flowers', name: 'Flowers' },
			{ id: 'glasses', name: 'Glasses' },
			{ id: 'beanie', name: 'Beanie' },
			{ id: 'crown', name: 'Crown' },
			{ id: 'bow', name: 'Bow' },
			{ id: 'earrings', name: 'Earrings' },
			{ id: 'monocle', name: 'Monocle' }
		]
	}
];

export const DEFAULT_PARTS: PartSelection = {
	head: 'peach',
	eyes: 'sparkle',
	nose: 'curve',
	mouth: 'berry',
	accessory: 'none'
};
