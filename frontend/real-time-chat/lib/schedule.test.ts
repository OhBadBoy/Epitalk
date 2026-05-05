import { describe, expect, it } from "vitest";

import {
  formatScheduledAt,
  getNextWeeklyOccurrenceLocal,
  toBackendUtcSchedule,
} from "./schedule";

describe("getNextWeeklyOccurrenceLocal", () => {
  it("returns a future occurrence within the next 7 days", () => {
    const now = new Date();
    const oneMinuteLater = new Date(now.getTime() + 60_000);

    const localDay = oneMinuteLater.getDay() === 0 ? 7 : oneMinuteLater.getDay();
    const next = getNextWeeklyOccurrenceLocal(
      now,
      localDay,
      oneMinuteLater.getHours(),
      oneMinuteLater.getMinutes(),
    );

    expect(next).not.toBeNull();
    expect((next as Date).getTime()).toBeGreaterThan(now.getTime());
    expect((next as Date).getTime() - now.getTime()).toBeLessThanOrEqual(7 * 24 * 60 * 60 * 1000);
  });

  it("returns null on invalid schedule values", () => {
    const now = new Date();

    expect(getNextWeeklyOccurrenceLocal(now, 0, 12, 0)).toBeNull();
    expect(getNextWeeklyOccurrenceLocal(now, 3, 24, 0)).toBeNull();
    expect(getNextWeeklyOccurrenceLocal(now, 3, 12, 60)).toBeNull();
  });
});

describe("toBackendUtcSchedule", () => {
  it("maps Sunday UTC day to 7", () => {
    const utcSunday = new Date("2026-04-19T14:45:00.000Z");
    const schedule = toBackendUtcSchedule(utcSunday);

    expect(schedule.dayOfWeek).toBe(7);
    expect(schedule.hour).toBe(14);
    expect(schedule.minute).toBe(45);
  });
});

describe("formatScheduledAt", () => {
  it("formats date in local timezone representation", () => {
    const date = new Date("2026-04-15T14:45:00Z");

    expect(formatScheduledAt(date, "en").length).toBeGreaterThan(0);
    expect(formatScheduledAt(date, "fr").length).toBeGreaterThan(0);
  });
});
