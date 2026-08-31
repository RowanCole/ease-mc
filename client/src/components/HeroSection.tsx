interface HeroSectionProps {
  title: string
  subtitle: string
}

export default function HeroSection({ title, subtitle }: HeroSectionProps) {
  return (
    <section className="hero-copy" aria-labelledby="launcher-title">
      <div className="eyebrow">
        <span /> WELCOME BACK
      </div>
      <h1 id="launcher-title">{title}</h1>
      <p>{subtitle}</p>
    </section>
  )
}
