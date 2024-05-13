import type { ServerInfo } from "~/api/model";

export const useServerInfo = () => useState<ServerInfo | undefined>('serverInfo');