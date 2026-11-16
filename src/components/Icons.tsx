import type { DeviceKind } from "../types";

type P = { size?: number };

const svg = (path: React.ReactNode, size = 16) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden>
    {path}
  </svg>
);

export const IconSearch = ({ size }: P) => svg(<><circle cx="11" cy="11" r="7" /><path d="m20 20-3.5-3.5" /></>, size);
export const IconRefresh = ({ size }: P) => svg(<><path d="M21 12a9 9 0 1 1-3-6.7L21 8" /><path d="M21 3v5h-5" /></>, size);
export const IconDownload = ({ size }: P) => svg(<><path d="M12 3v12" /><path d="m7 10 5 5 5-5" /><path d="M5 21h14" /></>, size);
export const IconSun = ({ size }: P) => svg(<><circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" /></>, size);
export const IconMoon = ({ size }: P) => svg(<path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8Z" />, size);
export const IconShield = ({ size }: P) => svg(<path d="M12 3 4 6v6c0 5 3.5 8 8 9 4.5-1 8-4 8-9V6l-8-3Z" />, size);
export const IconX = ({ size }: P) => svg(<path d="M18 6 6 18M6 6l12 12" />, size);
export const IconCopy = ({ size }: P) => svg(<><rect x="9" y="9" width="12" height="12" rx="2" /><path d="M5 15V5a2 2 0 0 1 2-2h10" /></>, size);
export const IconTable = ({ size }: P) => svg(<><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M3 10h18M9 4v16" /></>, size);
export const IconClock = ({ size }: P) => svg(<><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></>, size);
export const IconEject = ({ size }: P) => svg(<><path d="M12 4 4 14h16L12 4Z" /><path d="M4 19h16" /></>, size);
export const IconPower = ({ size }: P) => svg(<><path d="M12 3v9" /><path d="M6.3 7.3a8 8 0 1 0 11.4 0" /></>, size);
export const IconTrash = ({ size }: P) => svg(<><path d="M4 7h16M10 11v6M14 11v6" /><path d="M6 7l1 13h10l1-13M9 7V4h6v3" /></>, size);
export const IconSettings = ({ size }: P) => svg(<><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z" /></>, size);
export const IconBell = ({ size }: P) => svg(<><path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" /><path d="M10.3 21a1.9 1.9 0 0 0 3.4 0" /></>, size);
export const IconWindow = ({ size }: P) => svg(<><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M3 9h18" /></>, size);
export const IconInfo = ({ size }: P) => svg(<><circle cx="12" cy="12" r="9" /><path d="M12 11v6M12 7.5h.01" /></>, size);
export const IconExternal = ({ size }: P) => svg(<><path d="M14 4h6v6M20 4l-9 9" /><path d="M19 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h5" /></>, size);
export const IconGlobe = ({ size }: P) => svg(<><circle cx="12" cy="12" r="9" /><path d="M3 12h18M12 3c3 3.5 3 14.5 0 18M12 3c-3 3.5-3 14.5 0 18" /></>, size);

const kindPaths: Record<DeviceKind, React.ReactNode> = {
  storage: <><rect x="7" y="2" width="10" height="7" rx="1" /><rect x="5" y="9" width="14" height="13" rx="2" /><path d="M10 5h.01M14 5h.01" /></>,
  hid: <><rect x="2" y="6" width="20" height="12" rx="2" /><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10" /></>,
  audio: <><path d="M3 14v-2a9 9 0 0 1 18 0v2" /><rect x="3" y="14" width="4" height="7" rx="1" /><rect x="17" y="14" width="4" height="7" rx="1" /></>,
  video: <><rect x="2" y="7" width="14" height="10" rx="2" /><path d="m16 11 6-3v8l-6-3" /></>,
  phone: <><rect x="6" y="2" width="12" height="20" rx="2" /><path d="M11 18h2" /></>,
  printer: <><path d="M6 9V3h12v6" /><rect x="3" y="9" width="18" height="8" rx="2" /><rect x="7" y="14" width="10" height="7" /></>,
  network: <><rect x="9" y="2" width="6" height="6" rx="1" /><rect x="2" y="16" width="6" height="6" rx="1" /><rect x="16" y="16" width="6" height="6" rx="1" /><path d="M12 8v4M5 16v-4h14v4" /></>,
  bluetooth: <path d="m7 7 10 10-5 5V2l5 5L7 17" />,
  serial: <><path d="M4 8h16l-2 8H6L4 8Z" /><path d="M8 12h.01M12 12h.01M16 12h.01" /></>,
  hub: <><circle cx="12" cy="12" r="3" /><path d="M12 2v7M12 15v7M2 12h7M15 12h7" /></>,
  other: <><path d="M12 2v11" /><path d="m9 5 3-3 3 3" /><circle cx="12" cy="17" r="3" /><path d="M6 9v3l6 3 6-3V8" /></>,
};

export const KindIcon = ({ kind, size }: { kind: DeviceKind; size?: number }) => svg(kindPaths[kind], size);
