import { z } from "zod";

// Ce que l'utilisateur ENVOIE pour se connecter
export const LoginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(1),
});

export type LoginInput = z.infer<typeof LoginSchema>;

// Ce que l'utilisateur ENVOIE pour s'inscrire
// Le backend exige: min 8 chars, 1 majuscule, 1 chiffre, 1 caractère spécial
export const RegisterSchema = z.object({
  email: z.string().email("Email invalide"),
  username: z.string().min(3, "Minimum 3 caractères").max(32, "Maximum 32 caractères"),
  password: z
    .string()
    .min(8, "Minimum 8 caractères")
    .regex(/[A-Z]/, "Doit contenir au moins une majuscule")
    .regex(/[0-9]/, "Doit contenir au moins un chiffre")
    .regex(/[^A-Za-z0-9]/, "Doit contenir au moins un caractère spécial"),
});

export type RegisterInput = z.infer<typeof RegisterSchema>;

// Ce que le backend RENVOIE pour l'utilisateur
export const UserSchema = z.object({
  id: z.string(),
  username: z.string(),
  email: z.string(),
  avatar_url: z.string().nullable().optional(),
  bio: z.string().nullable().optional(),
  banner_color_1: z.string().nullable().optional(),
  banner_color_2: z.string().nullable().optional(),
  status: z.enum(["ONLINE", "IDLE", "DND", "OFFLINE"]).optional(),
  theme: z.string().optional(),
  created_at: z.string().optional(),
});

export type User = z.infer<typeof UserSchema>;

// Ce que le backend RENVOIE apres login/register
export const AuthResponseSchema = z.object({
  token: z.string(),
  token_type: z.string(),
  expires_in: z.number(),
  user: UserSchema,
});

export type AuthResponse = z.infer<typeof AuthResponseSchema>;
