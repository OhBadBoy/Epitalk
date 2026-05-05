"use client";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuthStore } from "@/store/auth.store";
import { authApi } from "@/lib/api";
import { useState } from "react";
import { getErrorMessage } from "@/lib/api/errors";
import { useLanguage } from "@/components/language-provider";

export function RegisterForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const setAuth = useAuthStore((state) => state.setAuth);
  const router = useRouter();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const { language } = useLanguage();
  const isEnglish = language === "en";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    // Validation côté client
    if (username.trim().length < 3) {
      setError(
        isEnglish
          ? "Username must be at least 3 characters long."
          : "Le nom d'utilisateur doit contenir au moins 3 caractères.",
      );
      return;
    }
    if (password.length < 8) {
      setError(
        isEnglish
          ? "Password must be at least 8 characters long."
          : "Le mot de passe doit contenir au moins 8 caractères.",
      );
      return;
    }

    setLoading(true);

    try {
      const response = await authApi.register({ email, username, password });
      setAuth(response);
      router.push("/servers");
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <Card>
        <CardHeader>
          <CardTitle>
            {isEnglish ? "Create your account" : "Créez votre compte"}
          </CardTitle>
          <CardDescription>
            {isEnglish
              ? "Enter your information to create a new account."
              : "Entrez vos informations pour créer un nouveau compte."}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit}>
            <FieldGroup>
              {error && (
                <div className="text-red-500 text-sm text-center p-2 bg-red-50 rounded">
                  {error}
                </div>
              )}

              <Field>
                <FieldLabel htmlFor="username">
                  {isEnglish ? "Username" : "Nom d'utilisateur"}
                </FieldLabel>
                <Input
                  id="username"
                  type="text"
                  placeholder="Nom d utilisateur"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  required
                  minLength={3}
                  maxLength={32}
                />
              </Field>

              <Field>
                <FieldLabel htmlFor="email">Email</FieldLabel>
                <Input
                  id="email"
                  type="email"
                  placeholder="m@example.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                />
              </Field>

              <Field>
                <FieldLabel htmlFor="password">
                  {isEnglish ? "Password" : "Mot de passe"}
                </FieldLabel>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  minLength={8}
                />
              </Field>

              <Field>
                <Button
                  className="bg-linear-to-r from-purple-700 to-orange-500 w-full"
                  type="submit"
                  disabled={loading}
                >
                  {loading
                    ? isEnglish
                      ? "Creating..."
                      : "Création..."
                    : isEnglish
                      ? "Create account"
                      : "Créer un compte"}
                </Button>

                <FieldDescription className="text-center">
                  {isEnglish ? "Already have an account? " : "Vous avez déjà un compte ? "}
                  <Link href="/login" className="underline">
                    {isEnglish ? "Sign in" : "Se connecter"}
                  </Link>
                </FieldDescription>
              </Field>
            </FieldGroup>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
