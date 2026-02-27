import 'vuetify/styles'
import '@mdi/font/css/materialdesignicons.css'
import { createVuetify } from 'vuetify'

export default createVuetify({
  theme: {
    defaultTheme: 'dark',
    themes: {
      dark: {
        colors: {
          primary: '#42A5F5',
          secondary: '#66BB6A',
          accent: '#FF7043',
        },
      },
      light: {
        colors: {
          primary: '#1565C0',
          secondary: '#2E7D32',
          accent: '#E64A19',
        },
      },
    },
  },
})
