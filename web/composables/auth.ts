import type { UserInfo } from "~/api/model";

export const useCurrentUser =  () => useState<UserInfo | undefined>('currentUser', () => undefined);
