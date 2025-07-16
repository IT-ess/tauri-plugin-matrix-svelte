import { z } from 'zod/v4';

const matrixUserId = z.string().regex(/^@[a-z0-9._=\-\/]+:[a-z0-9.\-]+\.[a-z]{2,}$/i, {
	message: 'Invalid Matrix user ID format. Must be: @localpart.server_name:port'
});

export const createDMRoomFormSchema = z.object({
	username: matrixUserId
});

export type CreateDMRoomFormSchema = typeof createDMRoomFormSchema;
