import { z } from 'zod';

export const loginFormSchema = z.object({
	username: z.string().min(2).max(50),
	password: z.string().min(8).max(100),
	homeserver: z.string().url().min(1).max(255),
	clientName: z.string().min(1).max(100)
});

export type LoginFormSchema = typeof loginFormSchema;
