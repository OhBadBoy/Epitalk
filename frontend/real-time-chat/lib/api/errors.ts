// Gestion centralisee des erreurs API
// Transforme les erreurs brutes du backend en messages lisibles en francais

// Erreur personnalisee que NOUS controlons
export class ApiError extends Error {
  status: number;
  code?: string;
  details?: unknown;
  fieldErrors?: Record<string, string[]>;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

type Language = "fr" | "en";

function getPreferredLanguage(): Language {
  if (typeof window === "undefined") return "fr";
  const stored = window.localStorage.getItem("epitalk_language");
  return stored === "en" ? "en" : "fr";
}

const ERROR_TRANSLATIONS_FR: Record<string, string> = {
  // Auth
  "Email already registered":          "Cette adresse email est deja utilisee.",
  "email already registered":          "Cette adresse email est deja utilisee.",
  "Username already taken":            "Ce nom d'utilisateur est deja pris.",
  "username already taken":            "Ce nom d'utilisateur est deja pris.",
  "Invalid credentials":               "Email ou mot de passe incorrect.",
  "invalid credentials":               "Email ou mot de passe incorrect.",
  "Invalid email or password":         "Email ou mot de passe incorrect.",
  "Unauthorized":                      "Vous devez etre connecte pour effectuer cette action.",
  "Token expired":                     "Votre session a expire. Veuillez vous reconnecter.",
  "Invalid token":                     "Session invalide. Veuillez vous reconnecter.",

  // Servers
  "Server not found":                  "Serveur introuvable.",
  "Not a member":                      "Vous n'etes pas membre de ce serveur.",
  "Already a member":                  "Vous etes deja membre de ce serveur.",
  "Only the owner can do this":        "Seul le proprietaire peut effectuer cette action.",
  "Cannot kick the owner":             "Impossible d'expulser le proprietaire.",
  "Forbidden":                         "Vous n'avez pas la permission d'effectuer cette action.",

  // Channels
  "Channel not found":                 "Channel introuvable.",
  "Channel name already exists":       "Un channel avec ce nom existe deja.",

  // Invites
  "Invite not found":                  "Invitation introuvable ou expiree.",
  "Invite expired":                    "Cette invitation a expire.",
  "Invite max uses reached":           "Cette invitation a atteint son nombre maximum d'utilisations.",
  "Invalid invite code":               "Code d'invitation invalide.",

  // Generics
  "Not found":                         "Ressource introuvable.",
  "Bad request":                       "Requete invalide.",
  "Internal server error":             "Erreur interne du serveur. Reessayez plus tard.",
};

const ERROR_TRANSLATIONS_EN: Record<string, string> = {
  // Auth
  "Email already registered": "This email address is already in use.",
  "email already registered": "This email address is already in use.",
  "Username already taken": "This username is already taken.",
  "username already taken": "This username is already taken.",
  "Invalid credentials": "Incorrect email or password.",
  "invalid credentials": "Incorrect email or password.",
  "Invalid email or password": "Incorrect email or password.",
  "Unauthorized": "You must be logged in to perform this action.",
  "Token expired": "Your session has expired. Please sign in again.",
  "Invalid token": "Invalid session. Please sign in again.",

  // Servers
  "Server not found": "Server not found.",
  "Not a member": "You are not a member of this server.",
  "Already a member": "You are already a member of this server.",
  "Only the owner can do this": "Only the owner can perform this action.",
  "Cannot kick the owner": "Cannot kick the owner.",
  "Forbidden": "You do not have permission to perform this action.",

  // Channels
  "Channel not found": "Channel not found.",
  "Channel name already exists": "A channel with this name already exists.",

  // Invites
  "Invite not found": "Invite not found or expired.",
  "Invite expired": "This invite has expired.",
  "Invite max uses reached": "This invite reached its maximum uses.",
  "Invalid invite code": "Invalid invite code.",

  // Generic
  "Not found": "Resource not found.",
  "Bad request": "Invalid request.",
  "Internal server error": "Internal server error. Please try again later.",
};

// Messages par defaut selon le code HTTP
const STATUS_MESSAGES_FR: Record<number, string> = {
  400: "Requete invalide. Verifiez les donnees saisies.",
  401: "Vous devez etre connecte pour effectuer cette action.",
  403: "Vous n'avez pas la permission d'effectuer cette action.",
  404: "Ressource introuvable.",
  409: "Un conflit est survenu. L'element existe peut-etre deja.",
  422: "Les donnees envoyees sont invalides.",
  429: "Trop de requetes. Veuillez patienter quelques instants.",
  500: "Erreur interne du serveur. Reessayez plus tard.",
  502: "Le serveur est momentanement indisponible.",
  503: "Service temporairement indisponible.",
};

const STATUS_MESSAGES_EN: Record<number, string> = {
  400: "Invalid request. Please check your input.",
  401: "You must be logged in to perform this action.",
  403: "You do not have permission to perform this action.",
  404: "Resource not found.",
  409: "A conflict occurred. The resource may already exist.",
  422: "Submitted data is invalid.",
  429: "Too many requests. Please wait a moment.",
  500: "Internal server error. Please try again later.",
  502: "Server is temporarily unavailable.",
  503: "Service temporarily unavailable.",
};

function translateError(raw: string, language: Language): string {
  const translations = language === "en" ? ERROR_TRANSLATIONS_EN : ERROR_TRANSLATIONS_FR;
  // Correspondance exacte
  if (translations[raw]) return translations[raw];
  // Correspondance insensible a la casse
  const lower = raw.toLowerCase();
  for (const [key, value] of Object.entries(translations)) {
    if (key.toLowerCase() === lower) return value;
  }
  // Correspondance partielle (le message contient une cle connue)
  for (const [key, value] of Object.entries(translations)) {
    if (lower.includes(key.toLowerCase())) return value;
  }
  return raw;
}

// Transforme la reponse backend en une erreur propre
export function parseApiError(
  status: number,
  payload: unknown
): ApiError {
  const language = getPreferredLanguage();
  let rawMessage = "";

  // Cas : backend renvoie un objet JSON
  if (typeof payload === "object" && payload !== null) {
    const obj = payload as Record<string, unknown>;
    // Format backend Rust : { "error": "...", "status": N }
    if (typeof obj.error === "string") {
      rawMessage = obj.error;
    }
    // Format alternatif : { "message": "..." }
    else if (typeof obj.message === "string") {
      rawMessage = obj.message;
    }
    // Format avec details
    else if (typeof obj.detail === "string") {
      rawMessage = obj.detail;
    }
  }

  // Cas : backend renvoie juste une string
  if (!rawMessage && typeof payload === "string" && payload.trim()) {
    try {
      const parsed = JSON.parse(payload);
      if (typeof parsed.error === "string") rawMessage = parsed.error;
      else if (typeof parsed.message === "string") rawMessage = parsed.message;
      else rawMessage = payload;
    } catch {
      rawMessage = payload;
    }
  }

  // Traduire le message
  const message = rawMessage
    ? translateError(rawMessage, language)
    : (language === "en" ? STATUS_MESSAGES_EN : STATUS_MESSAGES_FR)[status] ||
      (language === "en" ? `Error ${status}` : `Erreur ${status}`);

  return new ApiError(status, message);
}

// Helper pour l'UI - a utiliser dans les catch
export function getErrorMessage(error: unknown): string {
  const language = getPreferredLanguage();
  if (error instanceof ApiError) return error.message;
  if (error instanceof Error) {
    const msg = error.message;
    if (msg.startsWith("{")) {
      try {
        const parsed = JSON.parse(msg);
        const raw = parsed.error || parsed.message || msg;
        return translateError(raw, language);
      } catch {
        return msg;
      }
    }
    return msg;
  }
  return language === "en" ? "An error occurred." : "Une erreur est survenue.";
}
