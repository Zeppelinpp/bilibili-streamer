import { AppProvider } from '@/context/AppContext';

function App() {
  return (
    <AppProvider>
      <div className="flex h-screen bg-white text-stone-800 dark:bg-stone-950 dark:text-stone-200">
        <div className="flex-1 flex items-center justify-center">
          <span className="text-stone-400">App initialized</span>
        </div>
      </div>
    </AppProvider>
  );
}

export default App;
