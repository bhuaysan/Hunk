import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';

const target = document.getElementById('app');

if (!target) throw new Error('Hunk could not find its application root.');

mount(App, { target });
