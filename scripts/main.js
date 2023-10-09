const hashnodeBlogBaseUrl = "https://ifkash.hashnode.dev/";
const myPosts = [];

fetch("https://api.hashnode.com/", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    query: `{
      user(username: "ifkash") {
        publication {
          posts(page: 0) {
            slug
            title
          }
        }
      }
    }`,
  }),
})
  .then((res) => res.json())
  .then((res) => {
    const { posts } = res.data.user.publication;

    for (let i = 0; i < 2; ++i) {
      const { slug, title } = posts[i];
      const post = {
        slug: slug,
        title: title,
      };

      myPosts.push(post);
    }

    displayBlogPosts();
  })
  .catch((err) => console.error(err));


function createBlogPost(post) {
  const cardDiv = document.createElement("div");
  cardDiv.className = "card--project";

  const link = document.createElement("a");
  link.href = hashnodeBlogBaseUrl + post.slug;
  link.target = "_blank";
  link.rel = "noopener noreferrer";
  link.textContent = post.title;

  cardDiv.appendChild(link);

  return cardDiv;
}

function displayBlogPosts() {
  const myPostsContainer = document.getElementById("my-posts");
  
  if (!myPostsContainer) {
    console.error("Container with id 'my-posts' not found.");
    return;
  }
  
  const cardContainer = document.createElement("div");
  
  for (let i = 0; i < myPosts.length; i++) {
    const post = myPosts[i];
    const card = createBlogPost(post);
    cardContainer.appendChild(card);
  }

  myPostsContainer.appendChild(cardContainer);
}
