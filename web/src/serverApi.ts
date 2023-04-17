import axios, { Axios } from "axios";
import { LoginUser, MediaInfo, MediaSourceInfo, SearchResult, ServerInfo, SetupInfo, ToSetup, UserInfo } from "./models";
import { api } from "./request";

let cancelObj: any = {};
function cancelRequest(requestMethod: Function, identity: string | Function = 'default') {
    let id = typeof identity == 'function' ? identity.name : identity;
    if (cancelObj[requestMethod.name]) {
        cancelObj[requestMethod.name][id] && cancelObj[requestMethod.name][id]();
    }
}

function setCancelToken(tokenName: string | Function, identity: string | Function | undefined) {
    let name = typeof tokenName == 'string' ? tokenName as string : tokenName.name;
    if (identity) {
        let id = typeof identity == 'function' ? identity.name : identity;

        return new axios.CancelToken(c => {
            cancelObj[name] = {}
            cancelObj[name][id] = c;
        })
    }
    return undefined;
}

export async function getServerInfo(identity: string | Function | undefined = undefined): Promise<ServerInfo> {
    if (identity) cancelRequest(getServerInfo, identity);
    return (await api.get<ServerInfo>("/info", {
        cancelToken: setCancelToken(getServerInfo, identity)
    })).data;
}
export function getMediaFileAddress(id: string): string {
    return `/api/media_file/${id}`;
}

export async function getLibraries(identity: string | Function | undefined = undefined): Promise<MediaSourceInfo[]> {
    if (identity) cancelRequest(getLibraries, identity);
    let r = await api.get<MediaSourceInfo[]>(`/libraries`, {
        cancelToken: setCancelToken(getLibraries, identity)
    });
    return r.data;
}

export async function getAlbums(identity: string | Function | undefined = undefined): Promise<MediaSourceInfo[]> {
    if (identity) cancelRequest(getAlbums, identity);
    let r = await api.get<MediaSourceInfo[]>(`/albums`, {
        cancelToken: setCancelToken(getAlbums, identity)
    });
    return r.data;
}

export async function getCategories(identity: string | Function | undefined = undefined): Promise<MediaSourceInfo[]> {
    if (identity) cancelRequest(getCategories, identity); //here to continue
    let r = await api.get<MediaSourceInfo[]>(`/categories`, {
        cancelToken: setCancelToken(getAlbums, identity)
    });
    return r.data;
}

export async function getArtists(identity: string | Function | undefined = undefined): Promise<MediaSourceInfo[]> {
    if (identity) cancelRequest(getArtists, identity);
    let r = await api.get<MediaSourceInfo[]>(`/artists`, {
        cancelToken: setCancelToken(getArtists, identity)
    });
    return r.data;
}

export async function getGenres(identity: string | Function | undefined = undefined): Promise<MediaSourceInfo[]> {
    if (identity) cancelRequest(getGenres, identity);
    let r = await api.get<MediaSourceInfo[]>(`/genres`, {
        cancelToken: setCancelToken(getGenres, identity)
    });
    return r.data;
}

export async function getYears(identity: string | Function | undefined = undefined): Promise<MediaSourceInfo[]> {
    if (identity) cancelRequest(getYears, identity);
    let r = await api.get<MediaSourceInfo[]>(`/years`, {
        cancelToken: setCancelToken(getYears, identity)
    });
    return r.data;
}

export async function getLibraryInfo(title: string, identity: string | Function | undefined = undefined): Promise<MediaSourceInfo> {
    if (identity) cancelRequest(getLibraryInfo, identity);
    return (await api.get<MediaSourceInfo>(`/library_info?title=${title}`, {
        cancelToken: setCancelToken(getLibraryInfo, identity)
    })).data
}

export async function getAlbumInfo(title: string, identity: string | Function | undefined = undefined): Promise<MediaSourceInfo> {
    if (identity) cancelRequest(getAlbumInfo, identity);
    return (await api.get<MediaSourceInfo>(`/album_info?title=${title}`, {
        cancelToken: setCancelToken(getAlbumInfo, identity)
    })).data
}

export async function getCategoryInfo(title: string, identity: string | Function | undefined = undefined): Promise<MediaSourceInfo> {
    if (identity) cancelRequest(getCategoryInfo, identity);
    return (await api.get<MediaSourceInfo>(`/category_info?title=${title}`, {
        cancelToken: setCancelToken(getCategoryInfo, identity)
    })).data
}

export async function getArtistInfo(title: string, identity: string | Function | undefined = undefined): Promise<MediaSourceInfo> {
    if (identity) cancelRequest(getArtistInfo, identity);
    return (await api.get<MediaSourceInfo>(`/artist_info?title=${title}`, {
        cancelToken: setCancelToken(getArtistInfo, identity)
    })).data
}

export async function getGenreInfo(title: string, identity: string | Function | undefined = undefined): Promise<MediaSourceInfo> {
    if (identity) cancelRequest(getGenreInfo, identity);
    return (await api.get<MediaSourceInfo>(`/genre_info?title=${title}`, {
        cancelToken: setCancelToken(getGenreInfo, identity)
    })).data
}

export async function getYearInfo(title: string, identity: string | Function | undefined = undefined): Promise<MediaSourceInfo> {
    if (identity) cancelRequest(getYearInfo, identity);
    return (await api.get<MediaSourceInfo>(`/year_info?title=${title}`, {
        cancelToken: setCancelToken(getYearInfo, identity)
    })).data
}

export async function getMedias(source: string, filter: string, index: number, limit: number = 30, identity: string | Function | undefined = undefined): Promise<MediaInfo[]> {
    if (identity) cancelRequest(getMedias, identity);
    let r = await api.get<MediaInfo[]>(`/medias?source=${source}&filter=${filter}&index=${index}&limit=${limit}`, {
        cancelToken: setCancelToken(getMedias, identity)
    });
    return r.data;
}

export async function getMediaInfo(id: string, identity: string | Function | undefined = undefined): Promise<MediaInfo> {
    if (identity) cancelRequest(getMediaInfo, identity);
    let r = await api.get<MediaInfo>(`/media_info/${id}`, {
        cancelToken: setCancelToken(getMediaInfo, identity)
    });
    return r.data;
}

export async function searchMedia(toSearch: string, index: number, limit: number, identity: string | Function | undefined = undefined): Promise<SearchResult<MediaInfo>> {
    if (identity) cancelRequest(searchMedia, identity);
    let r = await api.get<SearchResult<MediaInfo>>(`/medias/search?content=${toSearch}&index=${index}&limit=${limit}`, {
        cancelToken: setCancelToken(searchMedia, identity)
    });
    return r.data;
}

export async function searchMediaFilter(toSearch: string, index: number, limit: number, source: string, filter: string, identity: string | Function | undefined = undefined): Promise<SearchResult<MediaInfo>> {
    if (identity) cancelRequest(searchMedia, identity);
    let r = await api.get<SearchResult<MediaInfo>>(`/medias/search?content=${toSearch}&index=${index}&limit=${limit}&source=${source}&filter=${filter}`, {
        cancelToken: setCancelToken(searchMedia, identity)
    });
    return r.data;
}

export async function getUser(username: string, identity: string | Function | undefined = undefined): Promise<UserInfo> {
    if (identity) cancelRequest(getUser, identity);
    let r = await api.get<UserInfo>(`/user/${username}`, {
        cancelToken: setCancelToken(getUser, identity)
    });
    return r.data;
}

export async function getUsers(index: number, limit: number = 30, identity: string | Function | undefined = undefined): Promise<UserInfo[]> {
    if (identity) cancelRequest(getUsers, identity);
    let r = await api.get<UserInfo[]>(`/users?index=${index}&limit=${limit}`, {
        cancelToken: setCancelToken(getUsers, identity)
    });
    return r.data;
}

export async function searchUser(toSearch: string, index: number, limit: number, identity: string | Function | undefined = undefined): Promise<SearchResult<UserInfo>> {
    if (identity) cancelRequest(searchUser, identity);
    return (await api.get<SearchResult<UserInfo>>(`/users/search?content=${toSearch}&index=${index}&limit=${limit}`, {
        cancelToken: setCancelToken(searchUser, identity)
    })).data;
}

export async function deleteUser(username: string, identity: string | Function | undefined = undefined) {
    await api.delete(`/user/${username}`, {
        cancelToken: setCancelToken(deleteUser, identity)
    });
}

export async function updateUser(to_update: UserInfo, identity: string | Function | undefined = undefined) {
    if (!to_update.password) throw new Error("Need password!");

    await api.put(`/user`, to_update, {
        cancelToken: setCancelToken(updateUser, identity)
    });
}

export async function createUser(to_create: UserInfo, identity: string | Function | undefined = undefined) {
    if (!to_create.password) throw new Error("Need password!");

    await api.post(`/user`, to_create, {
        cancelToken: setCancelToken(createUser, identity)
    });
}

export async function login(username: string, password: string, identity: string | Function | undefined = undefined): Promise<LoginUser> {
    if (identity) cancelRequest(login, identity);
    return (await api.get(`/login?username=${username}&password=${password}`, {
        cancelToken: setCancelToken(login, identity)
    })).data;
}

export async function logout(token: string, identity: string | Function | undefined = undefined): Promise<boolean> {
    if (identity) cancelRequest(logout, identity);
    return (await api.get(`/logout?token=${token}`, {
        cancelToken: setCancelToken(logout, identity)
    })).data;
}

export async function getCurrentUser(identity: string | Function | undefined = undefined): Promise<UserInfo> {
    if (identity) cancelRequest(getCurrentUser, identity);
    return (await api.get(`/current_user`, {
        withCredentials: true,
        cancelToken: setCancelToken(getCurrentUser, identity)
    })).data;
}

export async function setup(to_setup: ToSetup, identity: string | Function | undefined = undefined) {
    await api.post(`/setup`, to_setup, {
        cancelToken: setCancelToken(setup, identity)
    });
}

export async function getSetupInfo(identity: string | Function | undefined = undefined): Promise<SetupInfo> {
    if (identity) cancelRequest(getSetupInfo, identity);
    return (await api.get('/setup_info', {
        cancelToken: setCancelToken(getSetupInfo, identity)
    })).data;
}

export async function scanLibraries(identity: string | Function | undefined = undefined) {
    await api.get('/scan_libraries', {
        cancelToken: setCancelToken(scanLibraries, identity)
    });
}