<script setup lang="ts">
import { useUserStore } from "./stores/user";
import { listen } from "@tauri-apps/api/event";
import { AccessTokenResponse } from "./types";
import { useLogStore } from "./stores/logs";
import { requestNotificationPermission } from "./notify";

const logs = useLogStore();
const user = useUserStore();

listen<AccessTokenResponse>("update_access_token", (event) => {
  logs.clear();
  user.setAccessToken(event.payload);
});

if (user.isLoggedIn()) {
  user.validateToken();
}

requestNotificationPermission();
</script>

<template>
  <header class="mb-12 flex">
    <router-link to="/">
      <img
        src="./assets/lichess-white.svg"
        class="w-12 inline-block"
        alt="Lichess logo"
      />
    </router-link>

    <div class="grow">
      <nav class="flex space-x-4 justify-end">
        <router-link to="/" class="nav-item" active-class="active">
          Home
        </router-link>

        <router-link
          to="/broadcasts"
          class="nav-item"
          active-class="active"
          v-if="user.isLoggedIn()"
        >
          Broadcasts
        </router-link>

        <router-link to="/settings" class="nav-item" active-class="active">
          Settings
        </router-link>
      </nav>
    </div>
  </header>

  <router-view />
</template>
