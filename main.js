// ---- Scroll reveal animations ----
const animatedEls = document.querySelectorAll(
  '.service-card, .project-card, .client-tier, .credential-card'
);

if ('IntersectionObserver' in window) {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add('visible');
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.05, rootMargin: '0px 0px -20px 0px' }
  );
  animatedEls.forEach((el) => observer.observe(el));
} else {
  animatedEls.forEach((el) => el.classList.add('visible'));
}

// ---- Mobile nav toggle ----
const toggle = document.querySelector('.nav-toggle');
const mobileMenu = document.querySelector('.mobile-menu');

toggle.addEventListener('click', () => {
  mobileMenu.classList.toggle('open');
  const spans = toggle.querySelectorAll('span');
  if (mobileMenu.classList.contains('open')) {
    spans[0].style.transform = 'rotate(45deg) translate(5px, 5px)';
    spans[1].style.opacity = '0';
    spans[2].style.transform = 'rotate(-45deg) translate(5px, -5px)';
  } else {
    spans[0].style.transform = '';
    spans[1].style.opacity = '';
    spans[2].style.transform = '';
  }
});

// Close mobile menu on link click
mobileMenu.querySelectorAll('a').forEach((link) => {
  link.addEventListener('click', () => {
    mobileMenu.classList.remove('open');
    const spans = toggle.querySelectorAll('span');
    spans[0].style.transform = '';
    spans[1].style.opacity = '';
    spans[2].style.transform = '';
  });
});

// ---- Smooth scroll for anchor links ----
document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
  anchor.addEventListener('click', (e) => {
    const target = document.querySelector(anchor.getAttribute('href'));
    if (target) {
      e.preventDefault();
      target.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  });
});

// ---- Contact form ----
const contactForm = document.getElementById('contact-form');
const formStatus = document.getElementById('form-status');
const API_URL = 'https://zubago-contact.andy-235.workers.dev';

contactForm.addEventListener('submit', async (e) => {
  e.preventDefault();

  const btnText = contactForm.querySelector('.btn-text');
  const btnLoading = contactForm.querySelector('.btn-loading');
  const submitBtn = contactForm.querySelector('.btn-submit');

  // Get Turnstile token
  const turnstileToken =
    typeof turnstile !== 'undefined'
      ? turnstile.getResponse()
      : null;

  if (!turnstileToken && typeof turnstile !== 'undefined') {
    showStatus('Please complete the verification.', 'error');
    return;
  }

  // Disable button
  submitBtn.disabled = true;
  btnText.hidden = true;
  btnLoading.hidden = false;
  formStatus.hidden = true;

  const data = {
    name: contactForm.name.value,
    email: contactForm.email.value,
    subject: contactForm.subject.value,
    message: contactForm.message.value,
    turnstile_token: turnstileToken || '',
  };

  try {
    const res = await fetch(`${API_URL}/api/v1/contact`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.detail || 'Something went wrong. Please try again.');
    }

    showStatus('Message sent! We\'ll be in touch soon.', 'success');
    contactForm.reset();
    if (typeof turnstile !== 'undefined') turnstile.reset();
  } catch (err) {
    showStatus(err.message, 'error');
    if (typeof turnstile !== 'undefined') turnstile.reset();
  } finally {
    submitBtn.disabled = false;
    btnText.hidden = false;
    btnLoading.hidden = true;
  }
});

function showStatus(msg, type) {
  formStatus.textContent = msg;
  formStatus.className = `form-status ${type}`;
  formStatus.hidden = false;
}
