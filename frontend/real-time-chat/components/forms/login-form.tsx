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

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const setAuth = useAuthStore((state) => state.setAuth);
  const router = useRouter();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const { language } = useLanguage();
  const isEnglish = language === "en";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!email.trim() || !password.trim()) {
      setError(
        isEnglish ? "Please fill in all fields." : "Veuillez remplir tous les champs.",
      );
      return;
    }

    setLoading(true);

    try {
      const response = await authApi.login({ email, password });
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
            {isEnglish ? "Sign in to your account" : "Connectez-vous à votre compte"}
          </CardTitle>
          <CardDescription>
            {isEnglish
              ? "Enter your credentials to access your account."
              : "Entrez vos informations de connexion pour accéder à votre compte."}
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
                />
              </Field>

              <Field>
                <Button
                  type="submit"
                  className="bg-linear-to-r from-purple-700 to-orange-500 w-full"
                  disabled={loading}
                >
                  {loading
                    ? isEnglish
                      ? "Signing in..."
                      : "Connexion..."
                    : isEnglish
                      ? "Sign in"
                      : "Se connecter"}
                </Button>

                <FieldDescription className="text-center">
                  {isEnglish ? "Don’t have an account? " : "Vous n'avez pas de compte ? "}
                  <Link href="/register" className="underline">
                    {isEnglish ? "Sign up" : "S'inscrire"}
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
