"use client";

import { redirect } from "next/navigation";
import { useEffect } from "react";

interface RedirectProps {
  url: string;
}

export function Redirect({ url }: RedirectProps) {
  useEffect(() => redirect(url), [url]);

  return undefined;
}
