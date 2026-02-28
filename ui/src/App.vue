<template>
  <v-app>
    <v-app-bar density="compact" color="primary">
      <v-app-bar-nav-icon @click="drawer = !drawer" />
      <v-app-bar-title>
        <router-link to="/" class="text-white text-decoration-none">
          Harvex
        </router-link>
      </v-app-bar-title>
      <template #append>
        <v-btn icon to="/upload">
          <v-icon>mdi-upload</v-icon>
        </v-btn>
        <v-btn icon @click="toggleTheme">
          <v-icon>{{ isDark ? 'mdi-weather-sunny' : 'mdi-weather-night' }}</v-icon>
        </v-btn>
      </template>
    </v-app-bar>

    <v-navigation-drawer v-model="drawer" temporary>
      <v-list nav density="compact">
        <v-list-item
          prepend-icon="mdi-view-dashboard"
          title="Dashboard"
          to="/"
          exact
        />
        <v-list-item
          prepend-icon="mdi-upload"
          title="Upload"
          to="/upload"
        />
        <v-divider class="my-2" />
        <v-list-item
          prepend-icon="mdi-cog"
          title="Settings"
          to="/settings"
        />
      </v-list>
    </v-navigation-drawer>

    <v-main>
      <v-container fluid class="pa-4">
        <router-view />
      </v-container>
    </v-main>

    <!-- Global snackbar -->
    <v-snackbar
      v-model="snackbar.state.show"
      :color="snackbar.state.color"
      :timeout="snackbar.state.timeout"
      location="bottom right"
    >
      {{ snackbar.state.text }}
      <template #actions>
        <v-btn variant="text" @click="snackbar.hideSnackbar()">Close</v-btn>
      </template>
    </v-snackbar>
  </v-app>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTheme } from 'vuetify'
import { useSnackbar } from './composables/useSnackbar'

const theme = useTheme()
const snackbar = useSnackbar()
const drawer = ref(false)

const isDark = computed(() => theme.global.current.value.dark)

function toggleTheme() {
  theme.global.name.value = isDark.value ? 'light' : 'dark'
}
</script>
