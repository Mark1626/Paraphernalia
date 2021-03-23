import './App.css';
import { HelloWorld } from 'shared-components';

function App() {
  return (
    <div className="App">
      <header className="App-header">
        <span>Hello World from Main App</span>
        
        <span>Taking component from shared-library</span>
        <HelloWorld />

      </header>
    </div>
  );
}

export default App;
