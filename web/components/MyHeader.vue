<script setup lang="ts">
const currentUser = useCurrentUser();
const isOpen = ref(false)
const colorMode = useColorMode();
const toast = useToast();
const isDark = computed({
    get() {
        return colorMode.value === 'dark'
    },
    set(toDark) {
        colorModeLink.icon = toDark ? "i-heroicons-moon" : "i-heroicons-sun";
        colorMode.preference = toDark ? 'dark' : 'light'
    }
})
onMounted(() => {
    if (isDark.value) {
        colorModeLink.icon = "i-heroicons-moon"
    }
})

const titleLink = {
    label: '',
    badge: {
        color: 'primary',
        variant: 'subtle',
        class: 'py-0 text-2xl',
        label: "Diosic",
    },
    to: '/'
};

const colorModeLink = reactive({
    icon: "i-heroicons-sun",
    label: '',
    isSwitch: true,
    click() {
        isDark.value = !isDark.value;
    },
})

const sideLinks = [
    [
        {
            label: "Home",
            icon: 'i-heroicons-home',
            to: '/'
        },
        {
            label: "Libraries",
            icon: 'i-heroicons-inbox-stack',
            to: '/s/library'
        },
        {
            label: "Categories",
            icon: 'i-mdi-cards-heart-outline',
            to: '/s/category'
        },
        {
            label: "Albums",
            icon: 'i-mdi-album',
            to: '/s/album'
        },
        {
            label: "Artists",
            icon: 'i-mdi-account-music-outline',
            to: '/s/artist'
        },
        {
            label: "Genres",
            icon: 'i-mdi-format-list-bulleted-type',
            to: '/s/genre'
        },
        {
            label: "Years",
            icon: 'i-mdi-calendar-multiselect-outline',
            to: '/s/year'
        },
    ],
]

const unloginedSideLinks =
    [
        {
            label: "Login",
            icon: 'i-heroicons-user-circle',
            to: '/login'
        }
    ]

const loginedSideLinks = [
    ...sideLinks,
    [{
        label: "Workplace",
        icon: 'i-heroicons-table-cells',
        to: '/workplace'
    },{
        label: "Users",
        icon: 'i-heroicons-users',
        to: '/users'
    }, {
        label: "Logout",
        icon: 'i-heroicons-arrow-left-on-rectangle',
        async click() {
            toast.add({
                description: "Are you sure you want to log out now?",
                actions: [{
                    label: 'Yes!',
                    async click() {
                        await logoutNow();
                        location.replace("/login");
                    }
                }, {
                    label: 'No'
                }]
            })
        }
    }]
]

sideLinks.forEach(links => links.forEach((link: any) => {
    if (link.label === 'Logout') return;

    link.click = () => {
        isOpen.value = false;
    }
}))
loginedSideLinks.forEach(links => links.forEach((link: any) => {
    if (link.label === 'Logout') return;
    link.click = () => {
        isOpen.value = false;
    }
}))

const headerLinks = [
    [{
        label: '',
        icon: 'i-heroicons-bars-3-bottom-left-20-solid',
        iconClass: 'w-8',
        click() {
            isOpen.value = true
        }
    },
    titleLink as any],
    [
        colorModeLink,
    ]
]
</script>

<template>
    <div class="flex w-full">
        <div class="border-b border-neutral-300 dark:border-neutral-900 shadow-md w-full">
            <UHorizontalNavigation :ui="{ base: 'p-2' }" :links="headerLinks" />
        </div>

        <USlideover v-model="isOpen" prevent-close side="left">
            <UCard class="flex flex-col flex-1"
                :ui="{ body: { base: 'flex-1' }, ring: '', divide: 'divide-y divide-gray-100 dark:divide-gray-800' }">
                <template #header>
                    <div class="flex items-center justify-between p-2">
                        <NuxtLink class="text-base font-semibold leading-6 text-gray-900 dark:text-white" to="/">
                            {{ $config.public.forumName }}
                        </NuxtLink>
                        <UButton color="gray" variant="ghost" icon="i-heroicons-x-mark-20-solid"
                            class="-my-1 shadow-none" @click="isOpen = false" />
                    </div>
                </template>
                <UVerticalNavigation :links="currentUser ? loginedSideLinks : unloginedSideLinks" />
            </UCard>
        </USlideover>
    </div>
</template>