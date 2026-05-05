import { authApi } from "@/lib/api";
import { getErrorMessage } from "@/lib/api/errors";

export const dynamic = "force-dynamic";

export default async function TestPage() {
  let user: unknown = null;
  let errorMessage: string | null = null;

  try {
    user = await authApi.me();
  } catch (error) {
    errorMessage = getErrorMessage(error);
  }

  if (errorMessage) {
    return (
      <div style={{ padding: 20 }}>
        <h1>TEST API : KO ❌</h1>
        <p>{errorMessage}</p>
      </div>
    );
  }

  return (
    <div style={{ padding: 20 }}>
      <h1>TEST API : OK ! ✅</h1>
      <pre>{JSON.stringify(user, null, 2)}</pre>
    </div>
  );
}
