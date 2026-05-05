export function getNextWeeklyOccurrenceLocal(
  now: Date,
  dayOfWeek: number,
  hour: number,
  minute: number,
): Date | null {
  if (!(dayOfWeek >= 1 && dayOfWeek <= 7)) return null;
  if (!(hour >= 0 && hour <= 23)) return null;
  if (!(minute >= 0 && minute <= 59)) return null;

  const currentLocalDay = now.getDay(); // 0=Sun ... 6=Sat
  const targetLocalDay = dayOfWeek % 7; // 1..6 -> Mon..Sat, 0 -> Sun
  const dayDelta = (targetLocalDay - currentLocalDay + 7) % 7;

  const scheduled = new Date(now);
  scheduled.setDate(now.getDate() + dayDelta);
  scheduled.setHours(hour, minute, 0, 0);

  if (scheduled <= now) {
    scheduled.setDate(scheduled.getDate() + 7);
  }

  return scheduled;
}

export function toBackendUtcSchedule(date: Date): {
  dayOfWeek: number;
  hour: number;
  minute: number;
} {
  const utcDay = date.getUTCDay(); // 0=Sun ... 6=Sat
  return {
    dayOfWeek: utcDay === 0 ? 7 : utcDay,
    hour: date.getUTCHours(),
    minute: date.getUTCMinutes(),
  };
}

export function formatScheduledAt(date: Date, locale: "fr" | "en"): string {
  const formatter = new Intl.DateTimeFormat(locale, {
    weekday: "long",
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    timeZoneName: "short",
    hour12: false,
  });
  return formatter.format(date);
}
