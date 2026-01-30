import type { InReplyTo } from './InReplyTo.js';
import type { Replacement } from './Replacement.js';
import type { Thread } from './Thread.js';

export type Relation<C> =
	| { 'm.in_reply_to': InReplyTo }
	| ({ rel_type: 'm.replace' } & Replacement<C>)
	| Thread;
