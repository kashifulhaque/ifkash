
export const load = async ({ params, fetch }) => {
  const { slug } = params;
  const query = `
    query GetPost($slug: String!) {
      publication(host: "blog.ifkash.dev") {
        post(slug: $slug) {
          title
          subtitle
          publishedAt
          coverImage {
            url
          }
          content {
            html
          }
          tags {
            name
          }
        }
      }
    }
  `;

  try {
    const res = await fetch('https://gql.hashnode.com/', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        query,
        variables: { slug }
      })
    });

    if (!res.ok) {
      throw new Error('Failed to fetch post');
    }

    const result = await res.json();
    const post = result?.data?.publication?.post;

    if (!post) {
      // handle 404
      return {
        status: 404,
        error: new Error('Post not found')
      };
    }

    return {
      post
    };
  } catch (err) {
    console.error(err);
    return {
      status: 500,
      error: new Error('Error fetching post')
    };
  }
};
