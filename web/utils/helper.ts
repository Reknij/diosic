import { Source, type LoginQuery } from "~/api/model";
import { login, logout } from "~/api/user";
export const AUTH_COOKIE_NAME = "authorization";

export function parseSource(str: string): Source {
    const s = str.toLowerCase();
    if (s === "library") return Source.Library;
    if (s === "category") return Source.Category;
    if (s === "album") return Source.Album;
    if (s === "artist") return Source.Artist;
    if (s === "genre") return Source.Genre;
    if (s === "year") return Source.Year;

    return Source.Any;
}

export function toHHMMSS(sec_num: number) {
    const sec_num_floor = Math.floor(sec_num)
    const hours = Math.floor(sec_num_floor / 3600);
    const minutes = Math.floor((sec_num_floor - (hours * 3600)) / 60);
    const seconds = sec_num_floor - (hours * 3600) - (minutes * 60);

    return [hours, minutes, seconds]
        .map(v => v < 10 ? "0" + v : v)
        .filter((v, i) => v !== "00" || i > 0)
        .join(":");
}

export function setAuthQuery(url?: string) {
    const auth = useCookie(AUTH_COOKIE_NAME).value;
    if (url && auth) {
        return url.search(/\?{1}/) == -1 ? `${url}?auth=${auth}` : `${url}&auth=${auth}`
    }
    return url;
}


export async function loginNow(query: LoginQuery) {
    const { data, error } = await login(query);
    if (error.value) {
            const toast = useToast();
            if (error.value.statusCode === 404) {
            toast.add({
                color: 'red',
                title: 'Username or password is wrong!'
            })
        } else {
            toast.add({
                color: 'red',
                title: error.value.data
            })
        }
        console.error(error.value);
    } else if (data.value) {
        const expires = new Date(Date.now() + (1000 * 60 * 60 * 24 * 90));

        useCookie(AUTH_COOKIE_NAME, {
            expires
        }).value = data.value.token;
        const current = useCurrentUser();
        current.value = data.value.current;
    }
}

export async function logoutNow() {
    const token = useCookie(AUTH_COOKIE_NAME).value;
    if (!token) return;
    const { data, error } = await logout({
        token
    });
    if (error.value) {
        console.error(error.value);
    } else if (data.value) {
        useCookie(AUTH_COOKIE_NAME).value = undefined;
        const current = useCurrentUser();
        current.value = undefined;
    }
}