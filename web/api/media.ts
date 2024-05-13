import { useApiFetch } from "./customFetch";
import type { PubMediaInfo, GetMediasQuery, ListSlice } from "./model";

export function getMediaFileAddress(id: number): string {
    const auth = useCookie(AUTH_COOKIE_NAME).value;
    const origin = import.meta.dev? 'http://127.0.0.1:3177' : location.origin;
    return `${origin}/api/media_file/${id}?auth=${auth}`;
}

export async function getMedias(query: GetMediasQuery) {
    return useApiFetch<ListSlice<PubMediaInfo>>('/medias', {
        query
    })
}

export async function getMedia(id: number) {
    return useApiFetch<PubMediaInfo>(`/medias/${id}`)
}