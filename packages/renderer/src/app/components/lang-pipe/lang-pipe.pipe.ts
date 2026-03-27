import { Pipe, type PipeTransform } from '@angular/core';

@Pipe({
  name: 'langPipe',
})
export class LangPipePipe implements PipeTransform {
  transform(value: string): string {
    switch (value) {
      case 'en':
        return 'English 🇬🇧/🇺🇸';
      case 'es':
        return 'Spanish 🇪🇸';
      case 'et':
        return 'Estonian 🇪🇪';
      case 'fr':
        return 'French 🇫🇷';
      case 'de':
        return 'German 🇩🇪';
      case 'it':
        return 'Italian 🇮🇹';
      case 'nl':
        return 'Dutch 🇳🇱';
      case 'pl':
        return 'Polish 🇵🇱';
      case 'pt':
        return 'Portuguese 🇵🇹';
      case 'pt-BR':
        return 'Portuguese (Brazil) 🇧🇷';
      case 'ru':
        return 'Russian 🇷🇺';
      case 'tr':
        return 'Turkish 🇹🇷';
      case 'zh-CN':
        return 'Chinese 🇨🇳';
      case 'ja':
        return 'Japanese 🇯🇵';
      case 'ko':
        return 'Korean 🇰🇷';
      case 'id':
        return 'Indonesian 🇮🇩';
      case 'ar':
        return 'Arabic 🇸🇦';
      case 'am':
        return 'Amharic 🇪🇹';
      case 'eu':
        return 'Basque 🇪🇸';
      case 'hi':
        return 'Hindi 🇮🇳';
      case 'gl':
        return 'Galician 🇪🇸';
      case 'ro':
        return 'Romanian 🇷🇴';
      case 'sv':
        return 'Swedish 🇸🇪';
      case 'sl':
        return 'Slovenian 🇸🇮';
      case 'sw':
        return 'Swahili 🇰🇪';
      case 'hu':
        return 'Hungarian 🇭🇺';
      case 'uk':
        return 'Ukrainian 🇺🇦';
      case 'uz':
        return 'Uzbek 🇺🇿';
      default:
        return value;
    }
  }
}
