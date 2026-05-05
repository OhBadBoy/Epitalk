import { NextRequest, NextResponse } from "next/server";

const GIPHY_API_KEY = process.env.GIPHY_API_KEY;
const GIPHY_BASE = "https://api.giphy.com/v1/gifs";

export async function GET(req: NextRequest) {
  if (!GIPHY_API_KEY) {
    return NextResponse.json({ error: "GIF API not configured" }, { status: 503 });
  }

  const { searchParams } = new URL(req.url);
  const q = searchParams.get("q") || "trending";
  const limit = Math.min(Number(searchParams.get("limit") || "24"), 50);

  const endpoint =
    q === "trending"
      ? `${GIPHY_BASE}/trending?api_key=${GIPHY_API_KEY}&limit=${limit}&rating=g`
      : `${GIPHY_BASE}/search?api_key=${GIPHY_API_KEY}&q=${encodeURIComponent(q)}&limit=${limit}&rating=g`;

  const res = await fetch(endpoint);
  if (!res.ok) {
    return NextResponse.json({ error: "Giphy error" }, { status: res.status });
  }

  const json = await res.json();
  return NextResponse.json(json);
}
