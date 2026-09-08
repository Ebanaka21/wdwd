import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import HomePage from './pages/HomePage';
import ShowcasePage from './pages/ShowcasePage';
import AuthorDashboard from './pages/AuthorDashboard';
import UserDashboard from './pages/UserDashboard';
import AdminPanel from './pages/AdminPanel';
import LoginPage from './pages/LoginPage';

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route path="/:authorSlug" element={<ShowcasePage />} />
        <Route path="/author/dashboard" element={<AuthorDashboard />} />
        <Route path="/user/dashboard" element={<UserDashboard />} />
        <Route path="/admin" element={<AdminPanel />} />
        <Route path="/login" element={<LoginPage />} />
      </Routes>
    </Router>
  );
}

export default App;
