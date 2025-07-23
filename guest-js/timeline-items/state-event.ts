export type StateEvent =
	| { kind: 'otherState'; body: any }
	| { kind: 'membershipChange'; body: any }
	| { kind: 'profileChange'; body: any };
