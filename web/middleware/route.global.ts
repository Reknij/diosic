import { getServerInfo } from "~/api/server";
import { getCurrentUser } from "~/api/user";

async function initStates() {
    const { data: serverInfo } = await getServerInfo();
    if (serverInfo.value) {
        useServerInfo().value = serverInfo.value;
    }
    const { data: user } = await getCurrentUser();
    if (user.value) {
        useCurrentUser().value = user.value;
    }
}
export default defineNuxtRouteMiddleware(async (to, from) => {
    await callOnce(initStates);
    const serverInfo = useServerInfo();
    const currentUser = useCurrentUser();
    if (serverInfo.value?.admin_required) {
        if (to.path !== '/setup') {
            console.log("Navigate to setup because don't have admin");
            return navigateTo('/setup')
        }
    } else if (!currentUser.value && to.path !== '/login') {
            console.log("Navigate to login because no login.");
            return navigateTo('/login')
    }
})