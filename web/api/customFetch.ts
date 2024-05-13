type useFetchType = typeof useFetch;

export const useApiFetch: useFetchType = (path, opts = {}) => {
    const auth = useCookie(AUTH_COOKIE_NAME);
    const headers: any = {}
    if (auth.value) {
        headers["X-Authorization"] = auth.value.toString();
    }
    opts.baseURL = `/api`;
    opts.headers = headers;
    opts.key = `${path}${JSON.stringify(opts.query)}`;
    // opts.credentials = 'include'; // cloudflare workers will broken.
    return useFetch(path, opts);
};