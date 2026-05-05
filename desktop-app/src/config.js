function getWebAppUrl(env = process.env) {
  const fromEnv = env.EPITALK_WEB_URL;

  if (typeof fromEnv === "string" && fromEnv.length > 0) {
    if (/^https?:\/\//.test(fromEnv)) {
      return fromEnv;
    }
  }

  return "http://localhost:3000";
}

module.exports = {
  getWebAppUrl,
};
