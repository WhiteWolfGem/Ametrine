import { useState, useEffect } from 'react';
import './App.css';
import type { Post } from './types.ts';

function App() {
    const [posts, setPosts] = useState<Post[]>([]);
    const [activeTag, setActiveTag] = useState<string | null>(null);

    useEffect(() => {
        let url = 'http://localhost:3000/api/posts';
        if (activeTag) url += `?tag=${activeTag}`;

        fetch(url)
            .then((res) => res.json())
            .then((data) => setPosts(data))
            .catch((err) => console.error('Failed to fetch posts', err));
    }, [activeTag]);

    return (
        <div style={{ padding: '2rem', maxWidth: '800px', margin: '0 auto' }}>
            <h1>My Blog</h1>

            <hr />
            {posts.map((post) => (
                <article key={post.id} style={{ marginBottom: '2rem' }}>
                    <h2>{post.title}</h2>
                    <small>
                        {new Date(post.created_at).toLocaleDateString()}
                    </small>
                    <p>{post.content}</p>
                    <div>
                        {post.tags.map((tag) => (
                            <span
                                key={tag}
                                style={{ marginRight: '10px', color: '#666' }}
                                onClick={() => setActiveTag(tag)}
                            >
                                #{tag}
                            </span>
                        ))}
                    </div>
                </article>
            ))}
        </div>
    );
}

export default App;
