import React, { useState } from 'react';
import logo from './logo.svg';
import QrReader from 'react-qr-reader'
import './App.css';

function App() {
  const [result, setResult] = useState()
  const handleScan = (data) => {
    if(data) {
      setResult(data)
    }
  }

  const handleError = (err) => {
    console.log("Error", err)
  }

  return (
    <div className="App">
      <QrReader
          delay={300}
          onError={handleError}
          onScan={handleScan}
          style={{ width: '100%' }}
        />
        <p>{result}</p>
    </div>
  );
}

export default App;
