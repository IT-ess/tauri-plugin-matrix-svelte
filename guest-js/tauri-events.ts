// Emit events

export enum MatrixSvelteEmitEvent {
	RoomsCollectionStarted = 'matrix-svelte://rooms-collection-started',
	RoomCreated = 'matrix-svelte://room-created',
	VerificationResult = 'matrix-svelte://verification-result',
	UpdateCurrentActiveRoom = 'matrix-svelte://update-current-active-room'
}

export type RoomCreatedEventType = {
	id: string;
	message: string;
};

export type VerificationResultEventType = {
	confirmed: boolean;
};

export type UpdateCurrentActiveRoom = {
	roomId: string | null;
	roomName: string | null;
};

// Listen events

export enum MatrixSvelteListenEvent {
	RoomCreate = 'matrix-svelte://room-create',
	VerificationStart = 'matrix-svelte://verification-start',
	ToastNotification = 'matrix-svelte://toast-notification'
}

export type RoomCreateEventType = {
	id: string;
};

export type VerificationEmojisEventType = {
	emojis: string;
};

export type ToastNotificationEventType = {
	message: string;
	description: string | null;
	variant: ToastNotificationVariant;
};

type ToastNotificationVariant =
	| 'default'
	| 'description'
	| 'success'
	| 'info'
	| 'warning'
	| 'error';

// Channel events

export type MediaStreamEvent =
	| {
			event: 'started';
	  }
	| {
			event: 'chunk';
			data: {
				data: number[];
				chunkSize: number;
				bytesReceived: number;
			};
	  }
	| {
			event: 'finished';
			data: {
				totalBytes: number;
			};
	  }
	| {
			event: 'error';
			data: {
				message: string;
			};
	  };
